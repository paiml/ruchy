# Ruchy Development Server Example

This example demonstrates the Ruchy development server with hot reload and WASM compilation.

## Quick Start

```bash
# Navigate to this directory
cd examples/dev-server

# Start dev server with hot reload
ruchy serve --watch --watch-wasm --verbose

# Open browser to http://localhost:8080
# Edit main.ruchy and watch it auto-compile to main.wasm
```

## Features Demonstrated

### 1. Basic Static File Serving

```bash
# Serve the current directory
ruchy serve
```

Access at `http://localhost:8080/index.html`

### 2. Hot Reload on File Changes

```bash
# Enable watch mode
ruchy serve --watch
```

Try editing `index.html` or any file - server restarts automatically!

### 3. WASM Hot Reload

```bash
# Enable WASM compilation on save
ruchy serve --watch --watch-wasm
```

Workflow:
1. Edit `main.ruchy`
2. Save the file
3. Server detects change → compiles to `main.wasm`
4. Server restarts
5. Refresh browser to load new WASM

### 4. Custom Debounce

```bash
# Faster restarts (200ms debounce)
ruchy serve --watch --debounce 200

# Slower restarts (500ms debounce)
ruchy serve --watch --debounce 500
```

### 5. PID File Management

```bash
# Create PID file for process management
ruchy serve --watch --pid-file server.pid

# In another terminal, graceful shutdown:
kill -TERM $(cat server.pid)  # No more kill -9!
```

### 6. Network Access (Mobile Testing)

```bash
# Server shows both local and network URLs
ruchy serve --watch

# Output shows:
#   ➜  Local:   http://127.0.0.1:8080
#   ➜  Network: http://192.168.1.100:8080  ← Access from phone!
```

## File Structure

```
dev-server/
├── README.md          # This file
├── index.html         # Entry point
├── main.ruchy         # Ruchy source (auto-compiles to WASM)
├── styles.css         # Stylesheet
└── app.js             # JavaScript to load WASM
```

## Production Build

```bash
# 1. Compile Ruchy to WASM
ruchy wasm compile main.ruchy -o main.wasm

# 2. Serve without watch mode (production)
ruchy serve --port 8080
```

## Tips

- Use `--verbose` to see detailed compilation output
- Use `--debounce` to tune restart timing (default: 300ms)
- Press `Ctrl+C` for graceful shutdown
- Server displays both local and network URLs automatically
- WASM files are compiled in the same directory as source

## Troubleshooting

**Server won't restart after changes?**
- Increase debounce: `--debounce 500`
- Check file is in served directory

**WASM compilation fails?**
- Check Ruchy syntax with: `ruchy check main.ruchy`
- Run compilation manually: `ruchy wasm compile main.ruchy`

**Can't access from phone?**
- Ensure firewall allows port 8080
- Use network URL shown in server output
- Both devices must be on same network
