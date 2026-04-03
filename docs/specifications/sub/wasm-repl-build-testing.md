# Sub-spec: WASM REPL -- Build, Performance, and Testing

**Parent:** [wasm-repl-spec.md](../wasm-repl-spec.md) Build Configuration through Implementation Timeline Sections

---

## Build Configuration

```toml
[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
arrow2 = { version = "0.18", default-features = false, features = ["compute"] }

[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

# Custom allocator for minimal size
[dependencies.wee_alloc]
version = "0.4"
features = ["size_classes"]
```

```bash
#!/bin/bash
# build.sh

# Compile to WASM
cargo build --target wasm32-unknown-unknown --release

# Optimize with binaryen
wasm-opt -Oz \
    --enable-bulk-memory \
    --enable-mutable-globals \
    --converge \
    target/wasm32-unknown-unknown/release/ruchy_notebook.wasm \
    -o ruchy_notebook_opt.wasm

# Generate bindings
wasm-bindgen ruchy_notebook_opt.wasm \
    --out-dir pkg \
    --target web \
    --no-typescript

# Verify size
SIZE=$(stat -c%s pkg/ruchy_notebook_bg.wasm)
if [ $SIZE -gt 204800 ]; then
    echo "ERROR: WASM size ${SIZE} exceeds 200KB limit"
    exit 1
fi
```

## Performance Metrics

| Operation | Target | Implementation |
|-----------|--------|----------------|
| Bytecode compilation | <5ms | Achieved via single-pass compiler |
| Cell execution (simple) | <10ms | Stack VM with inline caching |
| Cell execution (1K DataFrame) | <50ms | Zero-copy Arrow operations |
| Memory per cell | 256KB | Arena reset between executions |
| Global promotion | <1ms | Explicit slab allocation |

## Enhanced DataFrame with Arrow Compute Kernels

### Apache Arrow DataFrame with Compute Kernels

```rust
use arrow::compute;
use arrow::array::{ArrayRef, Float64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

pub struct DataFrame {
    batch: RecordBatch,  // Arrow's native batch format
}

impl DataFrame {
    // Use Arrow's optimized kernels
    pub fn filter(&self, predicate: &BooleanArray) -> Result<DataFrame> {
        let filtered = compute::filter_record_batch(&self.batch, predicate)?;
        Ok(DataFrame { batch: filtered })
    }
    
    pub fn select(&self, columns: &[&str]) -> Result<DataFrame> {
        let schema = self.batch.schema();
        let indices: Vec<usize> = columns.iter()
            .filter_map(|name| schema.index_of(name).ok())
            .collect();
        
        let selected = self.batch.project(&indices)?;
        Ok(DataFrame { batch: selected })
    }
    
    pub fn group_by(&self, keys: &[&str]) -> GroupedDataFrame {
        // Use Arrow's aggregation kernels
        let key_columns: Vec<ArrayRef> = keys.iter()
            .map(|k| self.batch.column_by_name(k).unwrap().clone())
            .collect();
        
        GroupedDataFrame {
            source: self.batch.clone(),
            keys: key_columns,
        }
    }
    
    pub fn join(&self, other: &DataFrame, on: &str, how: JoinType) -> Result<DataFrame> {
        // Leverage Arrow's hash join
        let left_col = self.batch.column_by_name(on)?;
        let right_col = other.batch.column_by_name(on)?;
        
        let indices = compute::hash_join(left_col, right_col, how)?;
        let joined = self.merge_on_indices(&other, indices)?;
        
        Ok(DataFrame { batch: joined })
    }
    
    pub fn sort(&self, by: &[&str], ascending: bool) -> Result<DataFrame> {
        let options = compute::SortOptions {
            descending: !ascending,
            nulls_first: false,
        };
        
        let indices = compute::lexsort_to_indices(&self.batch, by, Some(options))?;
        let sorted = compute::take_record_batch(&self.batch, &indices)?;
        
        Ok(DataFrame { batch: sorted })
    }
}

pub struct GroupedDataFrame {
    source: RecordBatch,
    keys: Vec<ArrayRef>,
}

impl GroupedDataFrame {
    pub fn agg(&self, ops: Vec<AggOp>) -> Result<DataFrame> {
        let mut aggregated = Vec::new();
        
        for op in ops {
            let column = self.source.column_by_name(&op.column)?;
            
            let result = match op.function {
                AggFunc::Sum => compute::sum(column)?,
                AggFunc::Mean => compute::mean(column)?,
                AggFunc::Min => compute::min(column)?,
                AggFunc::Max => compute::max(column)?,
                AggFunc::Count => compute::count(column)?,
            };
            
            aggregated.push(result);
        }
        
        Ok(DataFrame::from_columns(aggregated))
    }
}
```

## Project Scaffolding

### Project Initialization

```rust
// CLI command for project setup
pub struct ProjectInit;

impl ProjectInit {
    pub fn create(path: &Path, template: ProjectTemplate) -> Result<()> {
        let project_root = path.canonicalize()?;
        
        // Create standard structure
        fs::create_dir_all(project_root.join("src"))?;
        fs::create_dir_all(project_root.join("notebooks"))?;
        fs::create_dir_all(project_root.join("data"))?;
        fs::create_dir_all(project_root.join("tests"))?;
        
        // Generate ruchy.toml
        let config = ProjectConfig {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            version: "0.1.0",
            entry: "src/main.ruchy",
            notebook_dir: "notebooks",
            dependencies: template.default_deps(),
        };
        
        let toml = toml::to_string_pretty(&config)?;
        fs::write(project_root.join("ruchy.toml"), toml)?;
        
        // Create example files based on template
        match template {
            ProjectTemplate::DataScience => {
                self.create_data_science_template(&project_root)?;
            }
            ProjectTemplate::WebService => {
                self.create_web_service_template(&project_root)?;
            }
            ProjectTemplate::Library => {
                self.create_library_template(&project_root)?;
            }
        }
        
        Ok(())
    }
    
    fn create_data_science_template(&self, root: &Path) -> Result<()> {
        // src/main.ruchy
        fs::write(root.join("src/main.ruchy"), r#"
// Data science project entry point
import "./data_prep.ruchy"
import "./analysis.ruchy"

global let config = {
    data_path: "data/",
    output_path: "results/"
}

fn main() {
    let data = load_dataset(config.data_path)
    let cleaned = prepare_data(data)
    let results = run_analysis(cleaned)
    save_results(results, config.output_path)
}
"#)?;
        
        // notebooks/exploration.ipynb
        let notebook = Notebook {
            cells: vec![
                Cell::Markdown("# Data Exploration\n\nInteractive analysis notebook".into()),
                Cell::Code("import \"../src/data_prep.ruchy\"\nlet df = load_dataset(\"../data/sample.csv\")".into()),
                Cell::Code("df.describe()".into()),
            ],
            metadata: NotebookMetadata::default(),
        };
        
        fs::write(
            root.join("notebooks/exploration.ipynb"),
            serde_json::to_string_pretty(&notebook)?
        )?;
        
        Ok(())
    }
}
```

### CLI Interface

```bash
# Initialize new project
$ ruchy init my_project --template data-science
Created project structure:
my_project/
├── ruchy.toml
├── src/
│   ├── main.ruchy
│   ├── data_prep.ruchy
│   └── analysis.ruchy
├── notebooks/
│   └── exploration.ipynb
├── data/
│   └── .gitkeep
└── tests/
    └── test_analysis.ruchy

# Run in different modes
$ cd my_project
$ ruchy run                    # Script mode: runs src/main.ruchy
$ ruchy repl                   # REPL mode: interactive session
$ ruchy notebook               # Notebook mode: opens browser
$ ruchy test                   # Runs tests/ directory

# Build for deployment
$ ruchy build --release        # Native binary
$ ruchy build --target wasm   # WASM module
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn bytecode_equivalence(code in any::<String>()) {
            let ast = parse(&code)?;
            let bytecode = compile(&ast)?;
            
            let interpreted = interpret(&ast)?;
            let executed = execute(&bytecode)?;
            
            prop_assert_eq!(interpreted, executed);
        }
        
        #[test]
        fn dataframe_zero_copy(size in 1..10000usize) {
            let df = DataFrame::random(size);
            let slice = df.slice(0, size/2);
            
            // Verify slicing doesn't copy data
            let df_ptr = df.columns[0].buffer().as_ptr();
            let slice_ptr = slice.columns[0].buffer().as_ptr();
            prop_assert_eq!(df_ptr, slice_ptr);
        }
    }
}
```

## Implementation Timeline

### Weeks 1-2: Bytecode VM
- OpCode design and stack machine
- Single-pass compiler from AST
- Basic arithmetic and control flow

### Weeks 3-4: Memory Management
- Arena allocator with reset
- Explicit global promotion
- Slab allocator for persistent values

### Weeks 5-6: Arrow DataFrame
- Buffer management and zero-copy slices
- Basic operations (select, filter, take)
- WASM serialization

### Weeks 7-8: Error Handling
- Structured error types
- Span tracking through compilation
- Suggestion engine

### Weeks 9-10: Demo Conversion
- Lightweight AST parser
- Statement grouping logic
- Batch conversion validation

### Weeks 11-12: Integration
- Frontend polish
- Performance optimization
- Documentation

