# Sub-spec: Ruchy Scoring — Unified Score Architecture and Interactive Debugger

**Parent:** [ruchy-scoring-spec.md](../ruchy-scoring-spec.md) Sections 13-17

---

### 5. Interactive Prover (`ruchy prove`)

REPL-based refinement type verification with developer-friendly tactics.

```rust
pub struct InteractiveProver {
    smt_solver: SmtSolver,
    proof_state: ProofState,
    tactic_library: TacticLibrary,
}

enum Tactic {
    Induction(Pattern),
    CaseAnalysis(Expression),
    Simplify,
    UseHypothesis(Name),
    Unfold(Definition),
    Auto, // Automated proof search
}

impl InteractiveProver {
    pub fn apply_tactic(&mut self, tactic: Tactic) -> ProofResult {
        let transformed = self.proof_state.apply(tactic);
        match self.smt_solver.check(transformed) {
            SmtResult::Proved => ProofResult::Success,
            SmtResult::Counterexample(cex) => {
                ProofResult::Failed(self.generate_test_case(cex))
            }
            SmtResult::Unknown => ProofResult::Incomplete(transformed)
        }
    }
    
    pub fn suggest_tactics(&self) -> Vec<(Tactic, f64)> {
        // ML-based tactic suggestion with confidence scores
        self.tactic_library.rank_by_similarity(&self.proof_state)
    }
    
    fn generate_test_case(&self, cex: Counterexample) -> String {
        // Convert SMT model to executable Ruchy test
        format!(
            "#[test]\nfn counterexample_{}_{}() {{\n    {}\n}}",
            self.proof_state.function_name,
            cex.hash(),
            cex.to_ruchy_code()
        )
    }
}
```

**Score Integration:**
```rust
// Proven properties dramatically increase correctness score
fn score_correctness_with_proofs(code: &Analysis, prover: &InteractiveProver) -> f64 {
    let base_score = score_correctness(code);
    let proof_coverage = prover.proven_properties() as f64 / prover.total_properties() as f64;
    
    // Proofs are weighted heavily - they provide mathematical certainty
    base_score * 0.6 + proof_coverage * 0.4
}
```

## Unified Score Architecture

All tools contribute to the unified quality score through specialized analyzers:

```rust
pub struct UnifiedScorer {
    base_analyzer: BaseAnalyzer,
    mechanical_sympathy: Option<MechanicalSympathyTuner>,
    actor_observatory: Option<ActorObservatory>,
    dataflow_debugger: Option<DataflowDebugger>,
    component_toolkit: Option<ComponentToolkit>,
    interactive_prover: Option<InteractiveProver>,
}

impl UnifiedScorer {
    pub fn calculate_comprehensive_score(&self, code: &Code) -> QualityScore {
        let mut components = self.base_analyzer.analyze(code);
        
        // Each specialized tool refines the score with additional insights
        if let Some(tuner) = &self.mechanical_sympathy {
            components.performance = components.performance * 0.8 + 
                                    tuner.hardware_efficiency(code) * 0.2;
        }
        
        if let Some(observatory) = &self.actor_observatory {
            components.safety = components.safety * 0.9 + 
                              observatory.actor_health(code) * 0.1;
        }
        
        if let Some(prover) = &self.interactive_prover {
            components.correctness = components.correctness * 0.6 + 
                                   prover.proof_coverage(code) * 0.4;
        }
        
        QualityScore::from_components(components)
    }
}
```

## Implementation Timeline

| Tool | Priority | Duration | Dependencies | Risk Mitigation |
|------|----------|----------|--------------|-----------------|
| Interactive Debugger | 1 | 4 months | Source map generation | Phase 1: Interpreter-only (2mo) |
| Actor Observatory | 2 | 3 months | Runtime instrumentation | Copy-on-observe, no blocking |
| Dataflow Debugger | 3 | 2 months | DataFrame API stability | Reuse breakpoint infrastructure |
| Component Toolkit | 4 | 4 months | WIT spec finalization | Leverage wasm-pack patterns |
| Interactive Prover | 5 | 6 months | SMT solver integration | Z3 bindings, defer custom solver |
| Mechanical Sympathy | 6 | 8 months | LLVM-MCA bindings | Start with static analysis only |

### Expertise Requirements

Per review feedback on "unicorn" team requirements, the implementation strategy:

1. **Core Team Focus**: Parser, type system, transpiler (Rust expertise required)
2. **Community Contributions**: Tool analyzers, MCP tools (domain expertise sufficient)
3. **External Dependencies**: PMCP for MCP, Z3 for SMT, LLVM for optimization
4. **Incremental Expertise**: Hire specialists as tools mature, not upfront

### Discipline Enforcement

Per "Foundation First" risk assessment:

```rust
// Compile-time quality gates prevent feature development until foundations complete
#[cfg(not(phase_0_complete))]
compile_error!("Phase 0 (parser, type system, test coverage) must be complete");

pub struct QualityGate {
    phase_0_metrics: Phase0Metrics,
    enforcement: EnforcementLevel,
}

impl QualityGate {
    pub fn check_phase_0(&self) -> Result<()> {
        // Hard requirements - no bypass possible
        assert!(self.phase_0_metrics.parser_coverage > 0.95);
        assert!(self.phase_0_metrics.type_system_complete);
        assert!(self.phase_0_metrics.cyclomatic_max < 10);
        assert!(self.phase_0_metrics.satd_count == 0);
        Ok(())
    }
}
```

## Success Metrics

Realistic targets acknowledging scope challenges:

- **Phase 0 Completion**: 6 months (non-negotiable blocker)
- **MVP Scoring**: Month 7 (AST metrics only)
- **Tool Integration**: 1 tool per quarter after MVP
- **Production Ready**: 24 months for core + 3 tools
- **Full Vision**: 5+ years (acknowledged, not promised)

## Ruchy Interactive Debugger (`ridb`)

The missing critical component for dynamic analysis: step-by-step execution control with complete state inspection.

### Core Architecture

```rust
pub struct RuchyDebugger {
    // Multi-tier execution backends
    interpreter_backend: InterpreterDebugger,
    compiled_backend: DwarfDebugger,
    jit_backend: Option<JitDebugger>,
    
    // Source mapping infrastructure
    source_mapper: SourceMapper,
    
    // Runtime expression evaluator
    eval_engine: ExpressionEvaluator,
    
    // Actor system integration
    actor_inspector: ActorInspector,
}

pub struct DebuggerState {
    breakpoints: BTreeMap<Location, Breakpoint>,
    call_stack: Vec<StackFrame>,
    local_vars: HashMap<String, Value>,
    actor_states: DashMap<ActorId, ActorSnapshot>,
}
```

### Source-Level Debugging

The fundamental challenge: maintaining source fidelity across transpilation.

```rust
impl SourceMapper {
    pub fn map_to_ruchy(&self, addr: MachineAddress) -> SourceLocation {
        // Three-level mapping: machine -> rust -> ruchy
        let rust_loc = self.dwarf_reader.addr_to_line(addr)?;
        let ruchy_loc = self.transpiler_map.rust_to_ruchy(rust_loc)?;
        ruchy_loc
    }
    
    pub fn inject_debug_info(&mut self, mir: &MIR) -> MIR {
        // Embed source locations at MIR level for preservation through optimization
        mir.instructions.map(|inst| {
            inst.with_debug_location(self.current_source_loc())
        })
    }
}
```

### Expression Evaluation

Runtime compilation within debugger context.

```rust
impl ExpressionEvaluator {
    pub fn eval(&self, expr: &str, context: &StackFrame) -> Result<Value> {
        // Parse expression in current lexical context
        let ast = self.parser.parse_expr(expr)?;
        
        // Type check against captured locals
        let typed_ast = self.type_checker.check_with_env(
            ast, 
            &context.type_environment()
        )?;
        
        // Execute via interpreter (fast path)
        self.interpreter.eval_in_context(typed_ast, context.locals())
    }
}
```

### Actor-Aware Debugging

Non-invasive concurrent system inspection.

```rust
impl ActorInspector {
    pub fn pause_actor(&self, id: ActorId) -> ActorSnapshot {
        // Copy-on-pause semantics
        let actor = self.registry.get(id)?;
        
        ActorSnapshot {
            state: actor.shadow_state.clone(),
            mailbox: actor.mailbox.snapshot(),
            reduction_count: actor.reductions.load(Ordering::Acquire),
            supervision_tree: self.get_supervision_context(id),
        }
    }
    
    pub fn trace_messages(&self, filter: MessageFilter) -> MessageStream {
        // Zero-copy message inspection via ring buffer
        self.message_bus
            .tap()
            .filter(move |msg| filter.matches(msg))
    }
}
```

### Phased Implementation

#### Phase 1: Interpreter Debugger (Month 1-2)
```rust
impl InterpreterDebugger {
    pub fn step(&mut self) -> StepResult {
        // Direct AST traversal - no source mapping needed
        let next_node = self.interpreter.next_node();
        
        if self.should_break_at(next_node) {
            StepResult::Breakpoint(self.capture_state())
        } else {
            self.interpreter.execute_node(next_node);
            StepResult::Continue
        }
    }
}
```

#### Phase 2: Compiled Code Debugger (Month 3-4)
```rust
impl DwarfDebugger {
    pub fn attach(&mut self, pid: ProcessId) -> Result<()> {
        // OS-level process control
        #[cfg(unix)]
        unsafe { ptrace::attach(pid)?; }
        
        // Load debug symbols
        self.dwarf_reader.load_symbols(pid)?;
        
        // Install breakpoint handlers
        self.install_signal_handlers()?;
        
        Ok(())
    }
    
    pub fn single_step(&mut self) -> Result<()> {
        #[cfg(unix)]
        unsafe { 
            ptrace::single_step(self.pid)?;
            self.wait_for_signal()?;
        }
        
        self.update_state()
    }
}
```

### Post-Mortem Debugging

Panic capture with full state preservation.

```rust
impl PanicHandler {
    pub fn capture_crash_state(&self) -> CrashDump {
        CrashDump {
            stack_trace: self.unwind_stack(),
            local_variables: self.capture_all_locals(),
            heap_snapshot: self.capture_heap(),
            actor_states: self.snapshot_all_actors(),
            timestamp: Instant::now(),
        }
    }
    
    pub fn enter_post_mortem(&self, dump: CrashDump) -> RidbSession {
        // Reconstruct debugging session from crash dump
        RidbSession::from_dump(dump)
    }
}
```

### Score Integration

Debuggability becomes a component of maintainability.

```rust
fn score_debuggability(code: &Analysis) -> f64 {
    let mut score = 1.0;
    
    // Penalize stripped symbols
    if !code.has_debug_symbols { score -= 0.3; }
    
    // Penalize macro-heavy code (harder to debug)
    score -= (code.macro_expansion_ratio * 0.2).min(0.2);
    
    // Reward explicit error handling (easier post-mortem)
    score += (code.result_types_ratio * 0.1).min(0.1);
    
    // Penalize async complexity (harder to trace)
    score -= (code.async_depth / 10.0).min(0.2);
    
    score.max(0.0)
}

// Integrate into maintainability score
fn score_maintainability_with_debug(code: &Analysis) -> f64 {
    let base = score_maintainability(code);
    let debug = score_debuggability(code);
    base * 0.85 + debug * 0.15
}
```

### CLI Interface

```bash
# Start with debugger
ruchy debug script.ruchy

# Attach to running process
ruchy debug --attach <pid>

# Post-mortem on panic
ruchy run script.ruchy --panic=debug

# Debug specific actor
ruchy debug script.ruchy --actor=worker-1

# Debug with expression watches
ruchy debug script.ruchy --watch="data.len()" --watch="state.phase"
```

### Debugger Commands

```
ridb> b 42                    # Breakpoint at line 42
ridb> b main if x > 100       # Conditional breakpoint
ridb> n                       # Next line
ridb> s                       # Step into
ridb> c                       # Continue
ridb> p variable              # Print variable
ridb> inspect dataframe       # Rich inspection
ridb> eval x * 2 + y         # Evaluate expression
ridb> bt                      # Backtrace
ridb> up/down                 # Navigate stack
ridb> actor ls                # List actors
ridb> actor pause worker-1    # Pause specific actor
ridb> actor trace worker-*    # Trace actor messages
ridb> save dump.ridb         # Save session
ridb> q                       # Quit
```

### Performance Considerations

Debug builds maintain <5% overhead through:

1. **Lazy symbol loading**: DWARF info loaded on-demand
2. **Incremental source mapping**: Cache hot paths
3. **Copy-on-write snapshots**: Zero-cost until accessed
4. **Ring buffer telemetry**: Lock-free message tracing

### Integration Points

The debugger integrates with all other tools:

- **Score**: Debuggability metrics feed maintainability score
- **Observatory**: Share actor telemetry infrastructure
- **Dataflow Debugger**: Unified breakpoint system
- **Prover**: Import counterexamples as debug scenarios
- **Mechanical Sympathy**: Show assembly alongside source

