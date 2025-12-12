# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in Ruchy, please report it responsibly:

1. **DO NOT** create a public GitHub issue
2. Email security concerns to: security@ruchy-lang.org
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes

## Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Fix Development**: Depends on severity
- **Disclosure**: Coordinated with reporter

## Security Measures

### Code Quality

- All code passes `cargo clippy -- -D warnings`
- No `unsafe` blocks in transpiled output (see ADR-003)
- Property-based testing with 10,000+ cases
- Mutation testing coverage ≥ 75%

### Dependencies

- Regular `cargo audit` checks
- Dependabot enabled for security updates
- Minimal dependency footprint

### Build Integrity

- Reproducible builds via `flake.nix`
- Locked dependencies in `Cargo.lock`
- CI/CD runs all security checks

## Known Security Considerations

### Parser

- Input size limits prevent DoS
- Recursion depth limits prevent stack overflow

### Transpiler

- No arbitrary code execution
- Generated Rust code is safe by design

### Oracle ML

- Model runs locally (no external API calls)
- No user data transmitted

## Audit History

| Date | Auditor | Scope | Result |
|------|---------|-------|--------|
| 2024-01-15 | Internal | Full codebase | Pass |
| 2024-06-01 | Internal | Dependency audit | Pass |
