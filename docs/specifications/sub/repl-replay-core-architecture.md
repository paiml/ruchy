# Sub-spec: REPL Replay Testing — Core Architecture

**Parent:** [repl-replay-testing-spec.md](../repl-replay-testing-spec.md) Section 2

---
## Core Architecture

### 1. Session Recording Format

```rust
#[derive(Serialize, Deserialize)]
pub struct ReplSession {
    version: SemVer,
    metadata: SessionMetadata,
    environment: Environment,
    timeline: Vec<TimestampedEvent>,
    checkpoints: BTreeMap<EventId, StateCheckpoint>,
}

#[derive(Serialize, Deserialize)]
pub struct TimestampedEvent {
    id: EventId,
    timestamp_ns: u64,
    event: Event,
    causality: Vec<EventId>, // Lamport clock for distributed replay
}

#[derive(Serialize, Deserialize)]
pub enum Event {
    Input { 
        text: String,
        mode: InputMode, // Interactive vs Paste vs File
    },
    Output {
        result: EvalResult,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    StateChange {
        bindings_delta: HashMap<Ident, Value>,
        type_env_delta: TypeEnvDelta,
        compilation_cache_delta: CacheDelta,
    },
    ResourceUsage {
        heap_bytes: usize,
        stack_depth: usize,
        cpu_ns: u64,
    },
}
```

### 2. Deterministic Execution Model

```rust
pub trait DeterministicRepl {
    fn execute_with_seed(&mut self, input: &str, seed: u64) -> ReplayResult;
    
    fn checkpoint(&self) -> StateCheckpoint;
    
    fn restore(&mut self, checkpoint: &StateCheckpoint) -> Result<()>;
    
    fn validate_determinism(&self, other: &Self) -> ValidationResult;
}

impl DeterministicRepl for Repl {
    fn execute_with_seed(&mut self, input: &str, seed: u64) -> ReplayResult {
        // Fix all sources of non-determinism
        self.rng.seed(seed);
        self.time_source = MockTime::new();
        self.allocator = DeterministicAllocator::new();
        
        let result = self.execute(input);
        
        ReplayResult {
            output: result,
            state_hash: self.compute_hash(),
            resource_usage: self.measure_resources(),
        }
    }
}
```

### 3. Replay Validation Engine

```rust
pub struct ReplayValidator {
    strict_mode: bool,
    tolerance: ResourceTolerance,
}

impl ReplayValidator {
    pub fn validate_session(&self, 
        recorded: &ReplSession, 
        implementation: &mut impl DeterministicRepl
    ) -> ValidationReport {
        let mut report = ValidationReport::new();
        let mut last_checkpoint = StateCheckpoint::initial();
        
        for event in &recorded.timeline {
            match event.event {
                Event::Input { ref text, .. } => {
                    let result = implementation.execute_with_seed(text, 0);
                    
                    // Validate output equivalence
                    let expected = recorded.next_output(event.id);
                    if !self.outputs_equivalent(&result.output, &expected) {
                        report.add_divergence(event.id, Divergence::Output);
                    }
                    
                    // Validate resource bounds
                    if !self.tolerance.accepts(&result.resource_usage) {
                        report.add_divergence(event.id, Divergence::Resources);
                    }
                    
                    // Store checkpoint for rollback testing
                    if recorded.checkpoints.contains_key(&event.id) {
                        last_checkpoint = implementation.checkpoint();
                    }
                }
                _ => {}
            }
        }
        
        report
    }
}
```

### 4. Differential Testing Harness

```rust
pub struct DifferentialTester {
    reference: ReferenceRepl,
    optimized: OptimizedRepl,
    session_generator: SessionGenerator,
}

impl DifferentialTester {
    pub fn test_equivalence(&mut self, iterations: usize) -> TestReport {
        let mut report = TestReport::new();
        
        for _ in 0..iterations {
            let session = self.session_generator.generate();
            
            let ref_result = replay_on(&mut self.reference, &session);
            let opt_result = replay_on(&mut self.optimized, &session);
            
            // Semantic equivalence, not bit-identical
            assert_semantic_eq!(ref_result, opt_result);
            
            // Performance must not regress
            assert!(
                opt_result.total_ns <= ref_result.total_ns * 1.1,
                "Performance regression detected"
            );
        }
        
        report
    }
}
```

