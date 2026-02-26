# Contributing to Ruchy

Contributions are welcome! Please follow these guidelines.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/<you>/ruchy.git`
3. Create a branch for your change

## Development

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Pull Requests

- All tests must pass (16,102+ tests)
- No clippy warnings
- Code must be formatted with `cargo fmt`
- Include tests for new functionality

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
