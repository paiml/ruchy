# ruchy-wasm

WebAssembly bindings for the [Ruchy programming language](https://github.com/paiml/ruchy).

## Features

- 🌐 **Browser-based compilation**: Compile Ruchy to Rust directly in the browser
- ⚡ **Fast validation**: Real-time syntax checking for interactive editors
- 📚 **Educational tools**: Perfect for documentation and learning platforms
- 🔧 **AST inspection**: Parse and analyze Ruchy code structure

## Installation

### NPM

```bash
npm install ruchy-wasm
```

### Building from Source

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build the WASM package
wasm-pack build --target web

# For bundler environments (webpack, vite, etc.)
wasm-pack build --target bundler
```

## Usage

### ES Modules (Vite, Modern Browsers)

```javascript
import init, { RuchyCompiler } from 'ruchy-wasm';

// Initialize the WASM module
await init();

// Create a compiler instance
const compiler = new RuchyCompiler();

// Compile Ruchy to Rust
const ruchyCode = `
struct Person {
    name: &str,
    age: i32
}

fn main() {
    let alice = Person { name: "Alice", age: 30 }
    println(alice.name)
}
`;

try {
    const rustCode = compiler.compile(ruchyCode);
    console.log(rustCode);
} catch (error) {
    console.error('Compilation failed:', error);
}
```

### Bundler (Webpack, Rollup)

```javascript
import init, { RuchyCompiler } from 'ruchy-wasm';

async function compileCode(source) {
    await init();
    const compiler = new RuchyCompiler();
    return compiler.compile(source);
}
```

### Syntax Validation

```javascript
import { RuchyCompiler } from 'ruchy-wasm';

const compiler = new RuchyCompiler();

// Validate syntax
const isValid = compiler.validate('let x = 42');
console.log('Valid syntax:', isValid); // true

const isInvalid = compiler.validate('let x = ');
console.log('Valid syntax:', isInvalid); // false
```

### AST Inspection

```javascript
import { RuchyCompiler } from 'ruchy-wasm';

const compiler = new RuchyCompiler();

// Get AST as JSON
const ast = compiler.parse_to_json('fn add(a, b) { a + b }');
console.log(JSON.parse(ast));
```

## API Reference

### `RuchyCompiler`

#### Constructor

```javascript
const compiler = new RuchyCompiler();
```

Creates a new Ruchy compiler instance.

#### Methods

##### `compile(source: string): string`

Compiles Ruchy source code to Rust.

- **Parameters**: `source` - Ruchy source code as a string
- **Returns**: Transpiled Rust code as a string
- **Throws**: Parse or transpile errors

##### `validate(source: string): boolean`

Validates Ruchy syntax without compilation.

- **Parameters**: `source` - Ruchy source code to validate
- **Returns**: `true` if syntax is valid, `false` otherwise

##### `parse_to_json(source: string): string`

Parses Ruchy code and returns AST as JSON.

- **Parameters**: `source` - Ruchy source code to parse
- **Returns**: AST representation as JSON string
- **Throws**: Parse errors

##### `version: string`

Returns the Ruchy compiler version.

## Examples

### Interactive Code Playground

```html
<!DOCTYPE html>
<html>
<head>
    <title>Ruchy Playground</title>
</head>
<body>
    <h1>Ruchy to Rust Compiler</h1>
    <textarea id="input" rows="10" cols="80">
fn fibonacci(n) {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
    </textarea>
    <button id="compile">Compile</button>
    <pre id="output"></pre>

    <script type="module">
        import init, { RuchyCompiler } from './pkg/ruchy_wasm.js';

        await init();
        const compiler = new RuchyCompiler();

        document.getElementById('compile').addEventListener('click', () => {
            const source = document.getElementById('input').value;
            try {
                const rust = compiler.compile(source);
                document.getElementById('output').textContent = rust;
            } catch (error) {
                document.getElementById('output').textContent = `Error: ${error}`;
            }
        });
    </script>
</body>
</html>
```

### Real-time Syntax Validation

```javascript
import { RuchyCompiler } from 'ruchy-wasm';

const compiler = new RuchyCompiler();
const editor = document.getElementById('code-editor');

editor.addEventListener('input', () => {
    const isValid = compiler.validate(editor.value);
    editor.classList.toggle('error', !isValid);
});
```

## Building for Interactive Books

Perfect for embedding in documentation sites like the [Ruchy Book](https://github.com/paiml/ruchy-book):

```javascript
// In your documentation build system
import { RuchyCompiler } from 'ruchy-wasm';

const compiler = new RuchyCompiler();

// Add "Try it" buttons to code blocks
document.querySelectorAll('code.language-ruchy').forEach(block => {
    const button = document.createElement('button');
    button.textContent = 'Compile';
    button.onclick = () => {
        try {
            const rust = compiler.compile(block.textContent);
            showResult(rust);
        } catch (e) {
            showError(e);
        }
    };
    block.appendChild(button);
});
```

## Size Optimization

The WASM binary is optimized for size:

- **opt-level = "s"**: Optimizes for binary size
- **LTO enabled**: Link-time optimization
- **Stripped symbols**: Removes debug information
- **Typical size**: ~500KB gzipped

## Browser Compatibility

- ✅ Chrome/Edge 57+
- ✅ Firefox 52+
- ✅ Safari 11+
- ✅ All modern browsers with WASM support

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.

## Links

- [Ruchy](https://github.com/paiml/ruchy) - Main repository
- [Ruchy Book](https://github.com/paiml/ruchy-book) - Learn Ruchy
- [crates.io](https://crates.io/crates/ruchy) - Rust package
