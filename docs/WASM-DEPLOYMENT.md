# Ruchy WASM Deployment Guide

## Overview

Ruchy provides WebAssembly binaries for browser and edge deployment, enabling interactive Ruchy execution directly in web applications.

**Version**: v3.99.2+
**Target**: `wasm32-unknown-unknown`
**Package Manager**: wasm-pack

## Quick Start

### Building WASM

```bash
# Using Makefile (recommended)
make wasm-build

# Or using wasm-pack directly
wasm-pack build --target web --no-default-features --features wasm-compile
```

**Output** (in `pkg/` directory):
- `ruchy_bg.wasm` - WebAssembly binary (~3.1MB)
- `ruchy.js` - JavaScript bindings
- `ruchy_bg.wasm.d.ts` - TypeScript definitions
- `package.json` - npm package metadata
- `README.md` - Package documentation

### Browser Usage

```html
<!DOCTYPE html>
<html>
<head>
    <title>Ruchy WASM REPL</title>
</head>
<body>
    <h1>Ruchy in the Browser</h1>
    <script type="module">
        import init, { WasmRepl } from './pkg/ruchy.js';

        async function main() {
            // Initialize WASM module
            await init();

            // Create REPL instance
            const repl = new WasmRepl();

            // Evaluate expressions
            const result1 = repl.eval('1 + 2');
            console.log(JSON.parse(result1));
            // { success: true, display: "3", timing: {...} }

            // String operations
            const result2 = repl.eval('"Hello" + " " + "World"');
            console.log(JSON.parse(result2));
            // { success: true, display: "Hello World", timing: {...} }

            // Function definitions
            const result3 = repl.eval('fn double(x) { x * 2 }; double(21)');
            console.log(JSON.parse(result3));
            // { success: true, display: "42", timing: {...} }
        }

        main();
    </script>
</body>
</html>
```

## REPL Output Format

The WASM REPL returns JSON with the following structure:

```typescript
interface ReplOutput {
    success: boolean;
    display?: string;        // Human-readable output (no quotes on strings)
    type_info?: string;      // Type information
    rust_code?: string;      // Generated Rust code (if applicable)
    error?: string;          // Error message (if success: false)
    timing: TimingInfo;
}

interface TimingInfo {
    parse_ms: number;        // Parsing time
    typecheck_ms: number;    // Type checking time (currently 0)
    eval_ms: number;         // Evaluation time
    total_ms: number;        // Total execution time
}
```

## WASM Limitations

Due to browser sandbox restrictions, the following features are **NOT available** in WASM builds:

### Excluded Operations
- **HTTP operations**: `http_get()`, `http_post()`, `http_put()`, `http_delete()`
- **File I/O**: `read_file()`, `write_file()`, file system operations
- **Process spawning**: Any system process execution
- **Native benchmarking**: Performance testing that requires native timing

### Conditional Compilation

These features are automatically gated using:
```rust
#[cfg(not(target_arch = "wasm32"))]
```

This ensures clean compilation for WASM targets without runtime errors.

## Deployment

### Manual Deployment

```bash
# Build and deploy to interactive.paiml.com
make wasm-deploy

# Or use the deployment script directly
./scripts/deploy-wasm.sh --all
```

### Release Process

The WASM build is integrated into the standard release process:

```bash
# Publish to crates.io + build WASM
make crate-release
```

This will:
1. Build WASM package with wasm-pack
2. Prompt for confirmation
3. Publish to crates.io
4. Display WASM artifacts location

### Deployment Script Options

```bash
# Build only (no deployment)
./scripts/deploy-wasm.sh --build

# Deploy existing build
./scripts/deploy-wasm.sh --deploy

# Build and deploy (default)
./scripts/deploy-wasm.sh --all
```

## Integration Examples

### React/Vue/Svelte

```typescript
import init, { WasmRepl } from 'ruchy-wasm';

let repl: WasmRepl | null = null;

async function initRuchy() {
    await init();
    repl = new WasmRepl();
}

function evaluate(code: string) {
    if (!repl) throw new Error('REPL not initialized');

    const result = JSON.parse(repl.eval(code));

    if (result.success) {
        console.log('Output:', result.display);
        console.log('Timing:', result.timing.total_ms, 'ms');
    } else {
        console.error('Error:', result.error);
    }
}
```

### Observable/Notebook Integration

```javascript
// Observable notebook cell
viewof replOutput = {
    const container = html`<div id="ruchy-repl"></div>`;
    const wasm = await import('./pkg/ruchy.js');
    await wasm.default();

    const repl = new wasm.WasmRepl();

    // Create interactive UI
    // ...

    return container;
}
```

## Testing

### E2E Tests

WASM functionality is validated using Playwright E2E tests:

```bash
# Install test dependencies
make e2e-install

# Run E2E tests (requires WASM build)
make test-e2e
```

### Manual Testing

```bash
# Start local server
cd pkg
python3 -m http.server 8080

# Open browser to http://localhost:8080
# Use browser console to test REPL
```

## Performance

**Binary Size**: ~3.1MB (uncompressed)
**Compression**: Gzip recommended (reduces to ~800KB)
**Load Time**: <500ms on modern connections
**Execution**: Near-native performance for most operations

**Note**: `wasm-opt` is disabled due to parse errors on large binaries. Future optimization may reduce binary size.

## Troubleshooting

### Build Failures

**Error**: `wasm-pack build` fails with compilation errors

**Solution**: Ensure you're using the correct feature flags:
```bash
wasm-pack build --target web --no-default-features --features wasm-compile
```

### Import Errors in Browser

**Error**: `Failed to fetch dynamically imported module`

**Solution**: Ensure files are served with correct MIME types:
- `.wasm` → `application/wasm`
- `.js` → `application/javascript`

### Runtime Errors

**Error**: `Function not found: http_get`

**Cause**: Attempting to use HTTP operations in WASM build

**Solution**: Use browser `fetch()` API instead:
```javascript
// Instead of: repl.eval('http_get("https://api.example.com")')
const response = await fetch('https://api.example.com');
const data = await response.text();
repl.eval(`let result = "${data}"`);
```

## Version History

### v3.99.2 (2025-10-21)
- ✅ Fixed WASM compilation errors
- ✅ Gated HTTP/file I/O operations
- ✅ Integrated WASM build into release process
- ✅ Added deployment automation scripts
- ✅ Fixed string evaluation (removed quotes from output)

### Previous Versions
- v3.99.1: WASM REPL partially working (67% tests passing)
- v3.75.0: Initial WASM support added

## Resources

- **WASM Specification**: [docs/specifications/wasm-repl-spec.md](specifications/wasm-repl-spec.md)
- **Quality Testing**: [docs/specifications/wasm-quality-testing-spec.md](specifications/wasm-quality-testing-spec.md)
- **E2E Tests**: [tests/e2e/notebook/](../tests/e2e/notebook/)
- **Interactive Demo**: https://interactive.paiml.com/wasm/ruchy/

## Contributing

When adding features that won't work in WASM:

1. Gate the code with `#[cfg(not(target_arch = "wasm32"))]`
2. Update tests to skip WASM-incompatible features
3. Document limitations in this file
4. Test WASM build: `make wasm-build`

## License

Same as Ruchy core: MIT License
