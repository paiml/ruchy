# Ruchy WASM Notebooks - Technical Specification v4.0

## Executive Summary

A browser-based notebook runtime for Ruchy, compiled to WebAssembly with bytecode interpretation. Delivers <50ms cell execution with Apache Arrow-compatible DataFrames in a <200KB WASM module.

---

## Sub-spec Index

| Sub-spec | Sections | Description |
|----------|----------|-------------|
| [wasm-repl-architecture.md](sub/wasm-repl-architecture.md) | Architecture | Bytecode VM, Parser, Type System, Robust Demo Conversion |
| [wasm-repl-state-language.md](sub/wasm-repl-state-language.md) | State & Language | Global State Management, Language Integration Model |
| [wasm-repl-build-testing.md](sub/wasm-repl-build-testing.md) | Build & Testing | Build Configuration, Performance Metrics, DataFrame, Project Scaffolding, Testing Strategy, Implementation Timeline |

---

## Conclusion

This specification delivers a performant notebook runtime through disciplined scope management and proven techniques. The bytecode VM provides 5-10x performance over tree-walking while remaining simple to implement. Apache Arrow compatibility ensures future interoperability without current complexity. Explicit global promotion eliminates hidden state dependencies.

The architecture scales from MVP to production without fundamental rewrites, validating the lean principle of building quality in from the start.

## Implementation Architecture

The notebook system ships as a subcommand of the main `ruchy` binary, not a separate installation. Single binary distribution maintains the zero-friction principle.

### Crate Structure

```
ruchy/
├── ruchy-core/          # Parser, type system, bytecode VM
├── ruchy-notebook/      # Notebook runtime (depends on core)
├── ruchy-wasm/          # WASM compilation target
└── ruchy-cli/           # Binary entry point
```

The `ruchy-notebook` crate compiles to both native and WASM targets. Native for local execution, WASM for browser deployment.

### Installation Experience

```bash
# Standard installation includes everything
$ cargo install ruchy
# or
$ brew install ruchy

# Verify notebook support
$ ruchy --version
ruchy 1.90.0 (notebook-enabled)
```

No separate tools. No npm packages. No Python environments. The notebook runtime is embedded in the main binary.

### Execution Modes

```bash
# Mode 1: Local notebook server (default)
$ ruchy notebook
Starting notebook server at http://localhost:8080
Opening browser...

# Mode 2: Convert and run existing script
$ ruchy notebook analysis.ruchy
Converted to notebook with 5 cells
Server running at http://localhost:8080

# Mode 3: Static WASM generation (no server)
$ ruchy notebook build analysis.ruchy --static
Generated: analysis.html (267KB, self-contained)

# Mode 4: Headless execution
$ ruchy notebook run analysis.ipynb --headless
Cell 1: ok
Cell 2: ok
Results saved to output/
```

### Runtime Architecture

The notebook server runs as a Rust web server (using `axum` or `warp`) that serves:
1. Static HTML/JS/CSS assets (embedded in binary)
2. WASM module (compiled from `ruchy-notebook`)
3. WebSocket endpoint for hot reload

```rust
// Embedded in ruchy-cli/src/notebook.rs
pub fn run_notebook_server(opts: NotebookOpts) -> Result<()> {
    let wasm_module = include_bytes!(concat!(env!("OUT_DIR"), "/notebook.wasm"));
    let frontend = include_str!("../assets/notebook.html");
    
    let app = Router::new()
        .route("/", get(|| async { Html(frontend) }))
        .route("/notebook.wasm", get(|| async { 
            Response::builder()
                .header("Content-Type", "application/wasm")
                .body(wasm_module.to_vec())
        }))
        .route("/ws", get(websocket_handler));
    
    Server::bind(&opts.addr).serve(app).await
}
```

### Browser Execution Flow

1. User navigates to `localhost:8080`
2. Browser loads minimal HTML shell (~5KB)
3. WASM module streams in (~200KB)
4. Notebook UI initializes
5. Code execution happens entirely client-side

No round-trips to server for execution. Server only handles file I/O and WebSocket for collaboration (future).

### File System Integration

```rust
// Native file access through server
#[wasm_bindgen]
impl NotebookRuntime {
    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>, JsValue> {
        // In browser: fetch from server
        let response = fetch(&format!("/files/{}", path)).await?;
        response.bytes().await
    }
}

// Server endpoint
async fn file_handler(Path(path): Path<String>) -> Result<Vec<u8>> {
    // Sandboxed to project directory
    let safe_path = sandbox_path(&path)?;
    tokio::fs::read(safe_path).await
}
```

### Migration Path for Existing Users

```bash
# Existing workflow unchanged
$ ruchy run script.ruchy  # Still works

# Progressive enhancement
$ ruchy notebook script.ruchy  # Opens as notebook

# Jupyter users
$ ruchy notebook import analysis.ipynb --from python
Converting Python notebook to Ruchy...
- pandas.read_csv() -> read_csv()
- df.groupby() -> df.group_by()
Conversion complete with 3 warnings
```

### Package Distribution

For users who want browser-only experience:

```bash
# Generate standalone package
$ ruchy notebook package my_analysis/
Creating my_analysis.zip:
  - index.html (5KB)
  - notebook.wasm (200KB)
  - notebook.js (12KB)
  - data/ (copied as-is)

# User opens index.html in browser
# Everything runs locally, no server needed
```

### VS Code Integration

```json
// .vscode/settings.json
{
  "ruchy.notebook.enabled": true,
  "ruchy.notebook.port": 8080
}
```

Command palette: "Ruchy: Open as Notebook" converts current file to notebook view within VS Code.

### Performance Characteristics

**Cold start**:
- Native: 50ms to first prompt
- Browser: 200ms to interactive (WASM init)

**Hot reload**:
- File change detected via `notify`
- WebSocket pushes update
- Cell re-executes in <10ms

**Memory overhead**:
- Server: ~20MB RSS
- Browser: ~50MB including WASM heap

### Production Deployment

For classroom/enterprise:

```bash
# Docker image with pre-built WASM
FROM rust:1.75 as builder
COPY . .
RUN cargo build --release --features notebook

FROM debian:slim
COPY --from=builder /target/release/ruchy /usr/local/bin/
EXPOSE 8080
CMD ["ruchy", "notebook", "--host", "0.0.0.0"]
```

Cloud deployment requires only static file hosting:

```bash
# Generate for CDN deployment
$ ruchy notebook build --static --base-url https://notebooks.example.com
$ aws s3 sync output/ s3://notebooks.example.com
```
