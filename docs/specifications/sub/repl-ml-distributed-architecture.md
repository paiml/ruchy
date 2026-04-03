# Sub-spec: REPL Magic — ML Training, Distributed Architecture & Operations

**Parent:** [repl-magic-spec.md](../repl-magic-spec.md) Sections 8-14

---


## Machine Learning Training Infrastructure

### Training Loop Primitives (Months 5-7)

#### Automatic Differentiation
```rust
#[autodiff]
fn loss(params: &Tensor, x: &Tensor, y: &Tensor) -> f32 {
    let pred = params.matmul(x);
    (pred - y).pow(2).mean()
}

// Reverse-mode AD generates gradient function
let grad_fn = loss.gradient();
```

Dual-number implementation for forward mode. Tape-based for reverse mode.

#### Training Loop Abstraction
```rust
trait Trainable {
    type Params: Tensor;
    type Input: Tensor;
    type Output: Tensor;
    
    fn forward(&self, x: &Self::Input) -> Self::Output;
    fn loss(&self, pred: &Self::Output, target: &Self::Output) -> f32;
}

// Automatic training loop generation
#[derive(Trainable)]
struct LinearModel {
    weights: Tensor2D,
    bias: Tensor1D,
}

>>> model.train(data, epochs=100, lr=0.01)
[████████████████████] Epoch 100/100, Loss: 0.0023
```

Training compiles to optimized loops. No Python overhead.

#### Distributed Training
```rust
#[distributed_training]
impl Model {
    fn train_step(&mut self, batch: Batch) -> Loss {
        // Automatically distributed via actor system
        let grads = self.backward(batch);
        
        // All-reduce across nodes
        all_reduce!(grads, Average);
        
        // Synchronized parameter update
        self.optimizer.step(grads);
    }
}

>>> model.train_distributed(
...     data: distributed_dataset,
...     nodes: cluster.gpus(),
...     strategy: DataParallel
... )
Distributing to 4 GPU nodes...
Node-1: Batch 0-250
Node-2: Batch 250-500
Node-3: Batch 500-750
Node-4: Batch 750-1000
[████████████] Global loss: 0.0019
```

Data parallel by default. Model parallel via annotations.

#### Hardware Acceleration
```rust
// CPU SIMD optimization
#[target_feature(enable = "avx512")]
fn matrix_multiply(a: &Matrix, b: &Matrix) -> Matrix

// GPU kernel generation (future)
#[gpu_kernel(block_size = 256)]
fn conv2d(input: &Tensor4D, kernel: &Tensor4D) -> Tensor4D

// Current approach: Candle backend
use candle::{Tensor, Device};

let device = Device::cuda_if_available();
let model = Model::new().to(device);
```

Candle provides CUDA/Metal support today. Custom kernels future.

#### Experiment Tracking
```rust
struct Experiment {
    id: Uuid,
    params: HashMap<String, Value>,
    metrics: TimeSeriesLog,
    artifacts: Vec<Artifact>,
}

#[track_experiment]
fn train_model(config: Config) -> Model {
    // Automatic logging
    log_params!(config);
    
    for epoch in 0..config.epochs {
        let loss = train_epoch();
        log_metric!("loss", loss);
        
        if epoch % 10 == 0 {
            log_artifact!(model.checkpoint());
        }
    }
}

>>> %experiments list
ID       | Loss  | Accuracy | Duration | Parameters
---------|-------|----------|----------|------------
exp-001  | 0.23  | 0.89     | 4m 23s   | lr=0.01, batch=32
exp-002  | 0.19  | 0.91     | 4m 45s   | lr=0.001, batch=64
exp-003  | 0.15  | 0.93     | 5m 12s   | lr=0.001, batch=128

>>> %experiments compare exp-001 exp-003
```

MLflow-compatible tracking. Local SQLite storage.

#### Model Serialization
```rust
// Zero-copy model serialization
impl Model {
    fn save(&self, path: &Path) -> Result<()> {
        // Weights as memory-mapped file
        let mmap = MmapMut::map_anon(self.size())?;
        self.serialize_into(&mut mmap[..])?;
        
        // Metadata as MessagePack
        let meta = ModelMeta {
            architecture: self.architecture(),
            optimizer_state: self.optimizer.state(),
            training_history: self.metrics,
        };
        meta.save(path.with_extension("meta"))?;
    }
}

>>> model.save("model.ruchy")
>>> let model = Model::load("model.ruchy").to(Device::Cuda(0))
```

ONNX export for interoperability. TensorRT optimization path.

#### Hyperparameter Optimization
```rust
#[hyperopt(method = "bayesian", trials = 100)]
fn train_with_config(
    #[param(min = 0.0001, max = 0.1, log = true)] lr: f64,
    #[param(min = 16, max = 256)] batch_size: usize,
    #[param(choices = ["adam", "sgd", "rmsprop"])] optimizer: String,
) -> f64 {
    let model = Model::new();
    model.train(config)?;
    model.validate()  // Returns loss
}

>>> let best_params = hyperopt::optimize(train_with_config)
[████████████] Trial 100/100, Best loss: 0.0012
Best parameters:
  lr: 0.0023
  batch_size: 128
  optimizer: "adam"
```

Optuna backend. Async trial execution. Pruning supported.

### Integration Points

#### With Actor System
- Distributed data loading
- Parameter servers
- Async gradient aggregation

#### With MCP
- Training progress to LLM
- Hyperparameter suggestions
- Architecture search

#### With WASM
- Browser-based training (limited)
- Model inference (full support)
- Federated learning nodes

## Distributed Architecture

### Actor System (Months 4-6)

#### Message Protocol
```rust
actor DataProcessor {
    state: ProcessorState,
    
    receive {
        Load(path) => {
            let data = read_csv(path)?;
            sender.reply(Ok(data));
        }
        Transform(df, ops) => {
            let result = ops.apply(df)?;
            sender.reply(Ok(result));
        }
    }
}
```

Erlang-inspired selective receive. Mailbox per actor. FIFO guarantee per sender.

#### Supervision
```rust
supervisor Pipeline {
    strategy: OneForOne,    // Isolate failures
    max_restarts: 3,
    window: 60s,
    
    children: [
        DataLoader { workers: 4 },
        Transformer { parallel: true },
        Writer { batch_size: 1000 }
    ]
}
```

Restart strategies: OneForOne, OneForAll, RestForOne. Exponential backoff standard.

#### Location Transparency
```rust
// Identical API
let local = spawn Actor::new()
let remote = spawn Actor::new() on "node-2"

// Transparent failover
select! {
    data = local.send(msg) => process(data),
    data = remote.send(msg) => process(data),
}
```

Actor references encapsulate location. Migration without code changes.

### MCP Integration

#### Tool Definition
```rust
#[mcp_tool("analyze")]
fn analyze(df: DataFrame) -> Analysis {
    Analysis {
        stats: df.describe(),
        correlations: df.corr(),
        outliers: detect_outliers(df)
    }
}
```

Compile-time schema extraction. Automatic OpenAPI generation.

#### Context Management
```rust
context DataScience {
    workspace: State,
    
    #[mcp_expose]
    fn datasets() -> Vec<DatasetInfo> {
        self.workspace.list_datasets()
    }
}
```

Bidirectional binding. Context updates propagate to LLM.

#### Protocol Validation
```rust
#[mcp_resource("segments")]
struct Segment {
    #[mcp_field(required)]
    id: String,
    
    #[mcp_field(min = 0.0, max = 100.0)]
    share: f64,
}
```

Static validation. Schema evolution tracked.

### Cluster Coordination

#### Node Management
```rust
%cluster connect ruchy://cluster
├── worker-1: 16 cores, 64GB
├── worker-2: 16 cores, 64GB
└── worker-3: 32 cores, 128GB

%cluster status
Node     | CPU | Memory  | Tasks | Status
---------|-----|---------|-------|-------
worker-1 | 45% | 12/64GB | 8     | Healthy
```

Gossip protocol for membership. Heartbeat interval 5s.

#### Distributed Execution
```rust
#[distributed]
fn process(df: DataFrame) -> Summary {
    df.chunks(10000)
      .parallel_map(analyze)
      .reduce(combine)
}
```

Automatic partitioning. Work-stealing scheduler.

## Implementation Timeline

### Q1: Foundation
- IPython/R/Julia compatibility
- Workspace persistence
- Polars integration
- Basic transpilation

**Exit Criteria**: 50ms REPL response, 100% command compatibility

### Q2: Enhancement & ML Basics
- Complete magics
- Modal interface
- Candle integration
- Basic training loops

**Exit Criteria**: MNIST training < 60s, pandas parity for 10 operations

### Q3: Distribution & WASM
- Actor runtime
- Supervision trees
- WASM compilation
- Browser notebook
- Distributed training

**Exit Criteria**: Linear scaling to 100 nodes, WASM notebook MVP

### Q4: Production & Advanced ML
- MCP protocol
- Hyperparameter optimization
- Experiment tracking
- Performance optimization
- Documentation

**Exit Criteria**: 5 production deployments, 1 ML model in production

## Architecture

### Component Stack
```
┌─────────────────────────────────────┐
│          Terminal Interface         │
├─────────────────────────────────────┤
│      Parser & Type Inference        │
├─────────────────────────────────────┤
│         Actor Runtime               │
├─────────────────────────────────────┤
│         MCP Protocol                │
├─────────────────────────────────────┤
│      Distributed Scheduler          │
├─────────────────────────────────────┤
│       Rust Crate Layer              │
│   (Polars, ndarray, linfa)          │
└─────────────────────────────────────┘
```

### Message Flow
```
Input → Parse → Type Check → Distribution Analysis
  ↓                              ↓
Local Execution          Actor Distribution
  ↓                              ↓
Result                   Aggregated Result
  ↓                              ↓
Display ← ─ ─ ─ Merge ─ ─ ─ ─ ─ ↲
```

## Performance Requirements

| Metric | Target | Maximum | Notes |
|--------|--------|---------|-------|
| REPL Response | 10ms | 50ms | P99 |
| DataFrame Op | 2x Polars | 5x | Includes transpilation |
| Actor Message | 0.1ms | 1ms | Local |
| Network Message | 5ms | 10ms | Cross-node |
| MCP Call | 50ms | 100ms | With validation |
| Cluster Join | 500ms | 2s | Full sync |

### Scalability
- Nodes: 100 (initial), 1000+ (future)
- Actors: 10,000 per node
- Messages: 1M/second cluster-wide
- DataFrame: 100GB+ distributed

## Testing Strategy

### Correctness
- Terminal compatibility suite
- Actor delivery guarantees
- MCP compliance tests
- Transpilation verification

### Performance
- Benchmarks vs Python/R/Julia
- Scalability to 100 nodes
- Actor stress tests
- Memory profiling

### Resilience
- Network partition tests
- Supervision recovery
- Chaos engineering

## Risk Analysis

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Actor complexity | Medium | High | Incremental implementation |
| Network reliability | High | Medium | QUIC, retry logic |
| Type inference | Medium | Medium | Conservative defaults |
| Adoption | High | High | Focus on Rust community |

## Competitive Position

Ruchy uniquely combines:
- Native actor model (vs library approach)
- Built-in MCP (vs external integration)
- Zero-cost abstractions (vs runtime overhead)
- Full terminal compatibility (vs new interface)

| Feature | Ruchy | Python+Ray | Julia+Distributed |
|---------|-------|------------|------------------|
| Actors | Native | Library | No |
| MCP | Native | No | No |
| Supervision | Yes | No | No |
| Type Safety | Complete | No | Optional |
| Performance | Native | Interpreted | JIT |

## Success Metrics

**Year 1 Goals**
- 1,000 active users
- 10 operations beating pandas
- 1M messages/second
- 5 production deployments

**Differentiation**
First language combining actor-based distribution with data science ergonomics and AI integration.

## Conclusion

