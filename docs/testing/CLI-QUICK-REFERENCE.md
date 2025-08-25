# CLI Testing Quick Reference

**🚀 Toyota Way: Quality Built-In - No Regressions Allowed**

## Essential Commands
```bash
# Run all CLI tests (733ms)
make test-ruchy-commands

# Check coverage (must be ≥80%)
make test-cli-coverage

# Performance benchmarking  
make test-cli-performance
```

## Test Structure
```
📋 Integration (8 tests)    → End-to-end scenarios
🔬 Property (5 tests)       → Mathematical invariants  
📦 Examples (4 scenarios)   → Executable documentation
🎲 Fuzz (2 targets)         → Random input robustness
```

## Coverage Standards
- **MINIMUM**: 80% (enforced by pre-commit)
- **CURRENT**: 87.80% ✅
- **TARGET**: 90% (aspirational)

## Performance Metrics ✅
| Component | Time | Target |
|-----------|------|---------|
| Integration | 176ms | <1s |
| Property | 193ms | <1s |
| **Total** | **733ms** | **<120s** |

## Adding New Commands
1. **Integration tests** → `tests/cli_integration.rs`
2. **Property tests** → `tests/cli_properties.rs` 
3. **Examples** → `examples/your_command_example.rs`
4. **Achieve ≥80% coverage**

## Quality Gates
- **Gate 16**: CLI coverage ≥80% (pre-commit hook)
- **All tests must pass**
- **Zero regressions policy**

---
**Never again will a CLI regression reach production. Quality is built-in, not bolted-on.**