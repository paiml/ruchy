# Ruchy - Self-Hosting Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/ruchy.svg)](https://crates.io/crates/ruchy)

Ruchy is a self-hosting programming language with comprehensive tooling and quality engineering practices based on Toyota Way principles.

## Quick Start

```bash
# Install from crates.io
cargo install ruchy

# Run a one-liner
ruchy -e "println('Hello, World!')"

# Start the REPL
ruchy repl

# Run a script
ruchy run script.ruchy

# Check syntax
ruchy check script.ruchy

# Format code
ruchy fmt script.ruchy

# Lint code
ruchy lint script.ruchy
```

## Key Features

### Self-Hosting Capability
- Bootstrap compiler written in Ruchy
- Transpiles to Rust for compilation
- Type inference with Algorithm W
- Complete language features for compiler development

### Language Features
- **Pipeline Operator**: `data |> transform |> filter`
- **Pattern Matching**: With guards: `x if x > 0 => "positive"`
- **Lambda Syntax**: Both `|x| x + 1` and `x => x + 1`
- **Module System**: `use`, `mod`, and `::` path resolution
- **Error Handling**: Result/Option types with `?` operator
- **Collections**: HashMap, HashSet with standard methods
- **String/Array Methods**: Comprehensive built-in methods

### CLI Commands
| Command | Purpose | 
|---------|---------|
| `ruchy check` | Syntax validation |
| `ruchy fmt` | Code formatting |
| `ruchy lint` | Quality analysis |
| `ruchy test` | Test execution |
| `ruchy ast` | AST visualization |
| `ruchy run` | Script execution |
| `ruchy repl` | Interactive environment |
| `ruchy transpile` | Convert to Rust |

### REPL Features
- Tab completion with context awareness
- Syntax highlighting
- Persistent history across sessions
- Multiline editing
- Magic commands (`:help`, `:load`, `:save`)
- Resource limits (memory, timeout, stack depth)

## Example Code

```rust
// Function definition
fun parse_expr(tokens: Vec<Token>) -> Result<Expr, ParseError> {
    match tokens.first() {
        Some(Token::Number(n)) => Ok(Expr::Literal(*n)),
        Some(Token::Ident(name)) => Ok(Expr::Variable(name.clone())),
        _ => Err(ParseError::UnexpectedToken)
    }
}

// Pipeline operator
[1, 2, 3, 4, 5]
  |> map(|x| x * 2)
  |> filter(|x| x > 5)
  |> sum()

// Pattern matching with guards
match user_input {
    n if n > 0 => "positive",
    0 => "zero",
    1..=10 => "small range",
    _ => "other"
}

// Collections
let mut map = HashMap()
map.insert("key", "value")
map.get("key").unwrap()
```

## Quality Engineering

### Code Quality Standards
- **Complexity Limits**: Functions must have cyclomatic complexity ≤10
- **Zero SATD Policy**: No TODO/FIXME/HACK comments allowed
- **Lint Compliance**: All clippy warnings treated as errors
- **Pre-commit Hooks**: Automated quality gates prevent regressions

### Testing
- Unit tests for core functionality
- Integration tests for CLI commands
- Property-based testing for mathematical invariants
- Fuzz testing for edge case discovery

### Quality Gate Script
```bash
# Run quality checks
./scripts/quality-gate.sh src

# Checks performed:
# - Function complexity ≤10
# - No technical debt comments
# - All tests passing
```

## Development

```bash
# Clone repository
git clone https://github.com/zacharycarter/ruchy.git
cd ruchy

# Build
cargo build --release

# Run tests
cargo test

# Check quality
./scripts/quality-gate.sh src

# Install locally
cargo install --path .
```

## Documentation

- [Language Specification](./SPECIFICATION.md)
- [Development Roadmap](./docs/execution/roadmap.md)
- [Change Log](./CHANGELOG.md)
- [Contributing Guidelines](./CLAUDE.md)

## Related Projects

- [ruchy-book](https://github.com/zacharycarter/ruchy-book) - Language documentation and examples
- [rosetta-ruchy](https://github.com/zacharycarter/rosetta-ruchy) - Algorithm implementations
- [ruchyruchy](https://github.com/zacharycarter/ruchyruchy) - Test suite

## License

MIT OR Apache-2.0