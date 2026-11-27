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

## Filing Issues

The repo caps open tickets at 50. Every new ticket is filed complete, in the same step as creation:

1. **Look for the group ticket first.** If an open group, master or train ticket covers the work, add a checklist line there and do not open a new ticket.
2. **File complete:** a parent (sub-issue of a master or epic), the parent's milestone, and the `triaged` label.
3. **At 50 open,** a new ticket is filed only by folding or closing another.
4. **Review follow-ups, CI reds and flake reports** go into the existing master as a checklist line, unless a PR will close them on its own.

## Pull Requests

- All tests must pass (16,102+ tests)
- No clippy warnings
- Code must be formatted with `cargo fmt`
- Include tests for new functionality

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
