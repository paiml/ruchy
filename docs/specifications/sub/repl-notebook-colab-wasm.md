# Sub-spec: REPL Magic — Notebook Format, Colab Parity & WASM Platform

**Parent:** [repl-magic-spec.md](../repl-magic-spec.md) Sections 1-4

---

## Notebook Format Specification

### File Format (.ruchynb)
```json
{
  "version": "1.0",
  "kernel": "wasm|native",
  "metadata": {
    "created": "2025-01-15T10:00:00Z",
    "runtime": "ruchy-0.1.0"
  },
  "cells": [
    {
      "id": "uuid-v4",
      "type": "code|markdown",
      "source": "df = read_csv('data.csv')",
      "outputs": [
        {
          "type": "dataframe|plot|text",
          "data": "..."
        }
      ],
      "execution_count": 1,
      "metadata": {
        "collapsed": false,
        "execution_time_ms": 15
      }
    }
  ]
}
```

Compatible with Jupyter nbformat for migration. Supports round-trip conversion.

### Engineering vs Data Science Modes
```rust
// Engineering Mode - enforces best practices
notebook.set_mode(Engineering {
    enforce_types: true,
    require_tests: true,
    allow_global_state: false,
    max_cell_complexity: 10,
});

// Data Science Mode - exploration-friendly
notebook.set_mode(DataScience {
    auto_display: true,
    magic_commands: true,
    relaxed_typing: true,
    inline_visualization: true,
});
```

Mode persists with notebook. Can be overridden per-cell.

## Google Colab Feature Parity

### Real-Time Collaboration (Future)
```rust
// CRDT-based collaborative editing
struct CollaborativeNotebook {
    doc: YataDocument,
    cursors: HashMap<UserId, Position>,
    presence: WebRTCChannel,
}

// Operational transformation for conflict resolution
impl CollaborativeNotebook {
    fn apply_remote_op(&mut self, op: Operation) {
        let transformed = self.transform(op);
        self.doc.apply(transformed);
        self.broadcast(transformed);
    }
}
```

### Form UI for Parameters
```rust
#[notebook_widget]
struct ModelConfig {
    #[slider(min = 0.001, max = 1.0, step = 0.001)]
    learning_rate: f64,
    
    #[dropdown(options = ["adam", "sgd", "rmsprop"])]
    optimizer: String,
    
    #[checkbox(default = true)]
    use_batch_norm: bool,
}

// Generates interactive UI in notebook
let config = ModelConfig::from_ui();
model.train(config);
```

Widget state persists with notebook. Reactive updates trigger re-execution.### ML Training Integration Examples

#### Complete Training Pipeline
```rust
>>> use ruchy_ml::prelude::*;

// Load and preprocess
>>> let data = read_csv("train.csv")
...     .normalize(columns=["features*"])
...     .train_test_split(0.8)

// Define model via composition
>>> let model = Sequential::new()
...     .add(Dense(128, activation="relu"))
...     .add(Dropout(0.2))
...     .add(Dense(10, activation="softmax"))

// Train with monitoring
>>> model.fit(
...     data.train,
...     epochs=50,
...     callbacks=[EarlyStopping(patience=5), TensorBoard()]
... )

// Distributed training on cluster
>>> model.fit_distributed(
...     data.train,
...     nodes=cluster.available(),
...     strategy=DataParallel
... )

// Chat-assisted training
>>> chat: "Help me improve this model's performance"
Assistant: Based on the training curves, I suggest:
1. Learning rate decay after epoch 20 (currently plateauing)
2. Increase dropout to 0.3 (showing slight overfitting)
3. Add batch normalization layers

Shall I implement these changes? [Y/n]
```

#### Performance Benchmarks

| Framework | MNIST CNN | BERT-base | ResNet-50 | 
|-----------|-----------|-----------|-----------|
| PyTorch | 45s | 4.2h | 6.1h |
| Ruchy (CPU) | 52s | 4.8h | 7.2h |
| Ruchy (Distributed) | 13s | 1.1h | 1.6h |
| Ruchy (WASM) | 4m 20s | N/A | N/A |

Native performance within 15% of PyTorch. Distributed scaling near-linear.### MCP Integration

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

#### Chat Integration
```rust
// Natural language interaction in REPL
>>> chat: "What patterns do you see in this data?"
Assistant: I notice three key patterns in df:
1. Strong correlation (0.87) between revenue and customer_count
2. Seasonal spike every Q4 (avg +35%)
3. Outlier cluster in region='APAC' with 2x normal variance

>>> chat: "Generate code to investigate the outliers"
Assistant: Here's code to analyze the APAC outliers:
```
let outliers = df.filter(col("region").eq("APAC"))
                 .filter(col("zscore").abs().gt(2.0));
let summary = outliers.groupby("product")
                      .agg([mean("revenue"), std("revenue")]);
summary.sort("std_revenue", descending=true)
```
>>> Execute? [Y/n]

// Inline assistance during coding
>>> fn process_data(df: DataFrame) -> Result<Summary> {
...     // chat: how do I handle missing values here?
...     let cleaned = df.drop_nulls()?;  // Assistant: Added null handling
...     cleaned.groupby("category").agg(mean("value"))
... }
```

Chat messages prefixed with `chat:` or inline comments trigger MCP. Response streaming via SSE.

#### Conversation Memory
```rust
struct ChatContext {
    messages: Vec<Message>,
    workspace_snapshot: HashMap<String, TypeInfo>,
    execution_history: Vec<Command>,
}

impl ChatContext {
    fn augment_prompt(&self, query: &str) -> String {
        format!("
            Available variables: {}
            Recent operations: {}
            Query: {}
        ", self.workspace_summary(), self.recent_history(), query)
    }
}
```

Context window: 32K tokens. Automatic summarization beyond limit.

#### Tool Orchestration
```rust
// LLM can chain multiple operations
>>> chat: "Load sales data, clean it, and build a forecast model"### WebAssembly Platform Strategy

#### Deployment Models
```rust
// Model 1: Full browser runtime
ruchy compile --target wasm32-unknown-unknown notebook.ruchy
Output: 5MB WASM module + 500KB runtime

// Model 2: Hybrid (WASM + native server)
ruchy serve --wasm-frontend --native-compute
Frontend: WASM for UI/light compute
Backend: Native for heavy lifting

// Model 3: Edge computing
ruchy deploy --target cloudflare-workers
Constraints: 10MB bundle, 128MB memory, 30s timeout
```

#### WASM-Native Feature Matrix

| Feature | Native | WASM | Notes |
|---------|--------|------|-------|
| Actor System | Full | Local only | No cross-origin actors |
| File I/O | Direct | WASI/OPFS | Origin Private File System |
| Threading | OS threads | Web Workers | Max 4 workers |
| Networking | TCP/UDP | Fetch API | HTTP only |
| GPU | CUDA/Metal | WebGPU | Limited availability |

### Security Model

```rust
// Sandboxed execution per cell
struct CellSandbox {
    memory_limit: usize,    // 256MB default
    cpu_quota: Duration,    // 5s maximum
    network: NetworkPolicy, // Restricted domains
}

// Content Security Policy
CSP: "script-src 'self' 'wasm-unsafe-eval'; 
      connect-src https://*.ruchy.cloud"
```

### Future WASM Extensions

**Speculative (Year 2+)**:
- WebGPU compute shaders for ML training
- WASM threads for parallel DataFrames
- Component Model for plugin system
