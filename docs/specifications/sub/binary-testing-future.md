# Sub-spec: Optimized Binary — Testing Strategy, Future Work, and References

**Parent:** [optimized-binary-speed-size-spec.md](../optimized-binary-speed-size-spec.md) Sections: Testing Strategy, Benchmark Suite, Documentation, Future Work, References

---


## Testing Strategy

### Regression Tests

1. **Profile Parity**: Verify all profiles produce working binaries
2. **Size Bounds**: Ensure `release-tiny` produces <500 KB binaries
3. **Performance Bounds**: Verify `release` achieves >10x speedup on standard benchmarks
4. **PGO Sanity**: Confirm PGO builds are faster than non-PGO
5. **Binary Analysis Validation** _(with `ruchy analyze`)_:
   - Verify size analysis matches expected profile characteristics
   - Confirm optimization recommendations are actionable
   - Validate format detection across platforms (ELF, Mach-O, PE)

### Integration Tests

```rust
#[test]
fn test_profile_release_default() {
    let output = compile_with_profile("test.ruchy", Profile::Release);
    assert!(output.size_kb() < 2000);  // < 2 MB
    assert!(output.runtime_ms() < baseline.runtime_ms() / 10);  // >10x
}

#[test]
fn test_profile_tiny_size() {
    let output = compile_with_profile("test.ruchy", Profile::ReleaseTiny);
    assert!(output.size_kb() < 500);  // < 500 KB
}

#[test]
fn test_analyze_size_breakdown() {
    let binary = compile_with_profile("test.ruchy", Profile::Release);
    let analysis = analyze_binary(&binary, AnalyzeOptions { size: true, ..Default::default() });

    // Verify .text section is dominant (>60% for code-heavy binaries)
    assert!(analysis.sections["text"].percentage > 60.0);

    // Verify total size matches expected range
    assert!(analysis.total_size_kb() >= 1000 && analysis.total_size_kb() <= 2000);
}

#[test]
fn test_analyze_optimization_recommendations() {
    let binary = compile_with_profile("test.ruchy", Profile::Release);
    let analysis = analyze_binary(&binary, AnalyzeOptions { optimize: true, ..Default::default() });

    // Should provide actionable recommendations
    assert!(!analysis.recommendations.is_empty());

    // Each recommendation should have impact estimate
    for rec in &analysis.recommendations {
        assert!(rec.impact_bytes > 0);
        assert!(rec.confidence >= 0.0 && rec.confidence <= 1.0);
    }
}
```

### Benchmark Suite

Run full pathfinder study on each Ruchy release:

```bash
# Generate benchmark workload in Ruchy
ruchy benchmark gen-workload --output bench.ruchy

# Compile with each profile
for profile in release release-tiny release-ultra; do
  ruchy compile bench.ruchy -o bench-$profile --profile $profile
  time ./bench-$profile
  ls -lh bench-$profile

  # Analyze binary characteristics
  ruchy analyze --size --optimize --output=analysis-$profile.json bench-$profile
done

# Compare results
ruchy analyze --size bench-release bench-tiny bench-ultra
```

**Acceptance Criteria**:
- `release`: >10x speedup, <2 MB, >60% .text section
- `release-tiny`: >2x speedup, <500 KB, optimized for size
- `release-ultra`: >15x speedup (with PGO), optimal symbol placement

**Binary Analysis Validation**:
```bash
# Verify size expectations match analysis
ruchy analyze --size bench-release --output=analysis.json
SIZE=$(jq '.total_size / 1024' analysis.json)  # KB
[ "$SIZE" -lt 2048 ] || exit 1  # Must be < 2 MB

# Verify optimization recommendations are reasonable
ruchy analyze --optimize bench-release --output=optim.json
SAVINGS=$(jq '.total_potential_savings_percent' optim.json)
# Should have <10% potential savings (already well-optimized)
[ "$(echo "$SAVINGS < 10" | bc -l)" -eq 1 ] || exit 1
```

---

## Documentation Updates

### README.md

```markdown
## Compilation Profiles

Ruchy uses research-backed compilation profiles for optimal binaries:

- **Default** (`ruchy compile`): 15x faster, 1-2 MB (best for most use cases)
- **Tiny** (`--profile release-tiny`): 2x faster, 300 KB (embedded systems)
- **Ultra** (`--profile release-ultra`): 25-50x faster (maximum performance)

Based on empirical data: [compiled-rust-benchmarking](https://github.com/paiml/compiled-rust-benchmarking)
```

### CLI Help Text

```
ABOUT PROFILES:
  Research shows LTO provides dramatic benefits (15x speed + 53% size reduction).
  All profiles use LTO by default. See our benchmarking study:
  https://github.com/paiml/compiled-rust-benchmarking
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Default speedup | >10x vs baseline | Benchmark suite |
| Tiny binary size | <500 KB | File size |
| User adoption | >80% use default | Telemetry (opt-in) |
| Compile time | <2min for 5K LOC | CI/CD metrics |
| Documentation clarity | >90% understand profiles | User survey |

---

## Future Work

### Advanced Optimization Features

1. **Auto-Profile Selection**: Analyze workload and recommend optimal profile
   - Use `ruchy analyze --optimize` to detect workload characteristics
   - Recommend `release` (speed), `release-tiny` (size), or `release-ultra` (max perf)

2. **Automated Optimization Application**:
   ```bash
   ruchy compile main.ruchy -o app --apply-recommendations
   # Automatically applies recommendations from binary analysis
   ```

3. **Cross-Language PGO**: Collect profiles from Ruchy execution, apply to Rust
   - Integration with `ruchy analyze --startup` for profile data collection

4. **Lazy Compilation**: JIT-compile hot paths discovered at runtime

5. **Binary Patching**: Update optimizations without full recompilation

6. **Interactive Optimization**:
   ```bash
   ruchy optimize main.ruchy
   # Interactive TUI showing:
   # - Current binary size breakdown
   # - Optimization recommendations
   # - Estimated impact of each change
   # - One-click apply
   ```

### Binary Analysis Enhancements _(Issue #145)_

1. **Startup Time Profiling**: Measure loader, linking, and init overhead
2. **Relocation Analysis**: Identify relocation hotspots and suggest static linking
3. **Symbol Deduplication**: Detect duplicate symbols across compilation units
4. **Comparative Analysis**: Compare against C/Rust/Go binaries for size benchmarking
5. **Machine Learning Recommendations**: Train model on successful optimizations
6. **Cache Profiling Integration**: Combine with perf events for cache-aware optimization
7. **HTML Report Generation**: Rich visual reports for optimization insights
8. **Flame Graph Support**: Visualize binary size contributions by module

### Research Extensions

1. **Workload Detection**: ML model to classify Ruchy program workload type
   - Integrate with `ruchy analyze` to auto-detect CPU/memory/I/O patterns

2. **Adaptive Optimization**: Runtime feedback to compiler for iterative improvement
   - Use `ruchy analyze --startup` for production profiling

3. **Cloud Profiling**: Collect PGO data from production deployments
   - Privacy-preserving aggregation of optimization data

4. **Size Budget Enforcement**:
   ```bash
   ruchy compile main.ruchy -o app --max-size=500KB
   # Automatically selects profile to meet constraint
   # Uses ruchy analyze to validate size target
   ```

---

## References

### Primary Research

- [Compiled Rust Benchmarking Infrastructure](https://github.com/paiml/compiled-rust-benchmarking)
  - 580 measurements across 10 workloads
  - 15 compilation profiles tested
  - Statistical validation (ANOVA, confidence intervals)

### Rust Documentation

- [Profile Settings](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Link-Time Optimization](https://doc.rust-lang.org/rustc/linker-plugin-lto.html)
- [Profile-Guided Optimization](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)

### Binary Analysis & Tooling

- [GitHub Issue #145](https://github.com/paiml/ruchy/issues/145) - `ruchy analyze` feature request
- [ruchyruchy COMPILED-INST-003](https://github.com/paiml/ruchyruchy) - Binary analysis prototype (6/6 tests passing)
- **goblin** crate: Cross-platform binary parsing (ELF, Mach-O, PE)
- **Bloaty McBloatface** (Google): Binary size profiler inspiration
- **cargo-bloat** (Rust): Cargo plugin for binary size analysis

### Related Specifications

- [PERF-001] - Speed-first optimization (v3.174.0)
- [PERF-002] - This document (Optimized binary speed & size)
- [Binary Build Story](binary-build-story.md) - Distribution strategy
- [Cargo Integration](cargo-integration-ruchy.md) - Rust toolchain integration

---

## Approval

- [ ] Engineering Lead Review
- [ ] Performance Team Approval
- [ ] Documentation Team Approval
- [ ] User Experience Review
- [ ] Security Audit (PGO data handling)

**Estimated Implementation**: 2 sprints (v3.212.0 - v3.214.0)

---

**Document Status**: Ready for Review
**Next Steps**: Team review, approve Phase 1 (documentation updates)
