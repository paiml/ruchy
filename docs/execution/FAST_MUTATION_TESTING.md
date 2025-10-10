# FAST Mutation Testing Strategy for Stdlib Modules

## Problem

Initial mutation testing approach ran ALL tests (3662 lib tests + integration tests) for each mutant:
- **Baseline timeout**: 95s build + >300s test = >395s total
- **Result**: Mutation testing IMPOSSIBLE - exceeded 300s timeout
- **Root cause**: Running thousands of unrelated tests for stdlib module mutations

## Solution: FAST Mutation Testing

**Strategy**: Run ONLY the specific integration test file relevant to the module being mutated.

### Configuration

File: `.cargo/mutants.toml`
```toml
# cargo-mutants configuration - FAST stdlib mutation testing
# Strategy: Run ONLY the specific integration test file for each module
#
# Usage (pass test name after --):
#   cargo mutants --file src/stdlib/fs.rs -- --test std_001_fs
#   cargo mutants --file src/stdlib/http.rs -- --test std_002_http
#   cargo mutants --file src/stdlib/json.rs -- --test std_003_json
#   cargo mutants --file src/stdlib/path.rs -- --test std_004_path
#
# This runs ~20 tests per module instead of 3662 lib tests
# Result: 5-10 minutes per module instead of hours
timeout_multiplier = 3.0
minimum_test_timeout = 120
```

### Command Syntax

**Key insight**: Use `--` to pass test args to `cargo test`:

```bash
cargo mutants --file src/stdlib/MODULE.rs -- --test TEST_FILE
```

NOT:
```bash
cargo mutants --file src/stdlib/MODULE.rs --test TEST_FILE  # WRONG
```

### Performance Comparison

| Approach | Test Count | Baseline Time | Total Runtime | Result |
|----------|-----------|---------------|---------------|--------|
| **ALL tests** | 3662 lib + integration | 95s build + >300s test | TIMEOUT | ❌ Failed |
| **FAST (--test)** | 16-20 integration tests | 101s build + 0.3s test | 7m 40s | ✅ Success |

**Performance gain**: Hours → Minutes (>90% reduction)

## Results

### STD-001 (fs module) - COMPLETE ✅
- **Command**: `cargo mutants --file src/stdlib/fs.rs -- --test std_001_fs`
- **Runtime**: 7m 40s
- **Mutants**: 18 total (16 caught, 2 unviable, 0 missed)
- **Coverage**: 100% (16/16 catchable mutations caught)
- **Status**: ✅ COMPLETE (exceeds ≥75% target)

### STD-002 (http module) - IN PROGRESS ⏳
- **Command**: `cargo mutants --file src/stdlib/http.rs -- --test std_002_http`
- **Expected runtime**: ~7-10 minutes
- **Mutants**: 12 found
- **Status**: Running

### STD-003 (json module) - IN PROGRESS ⏳
- **Command**: `cargo mutants --file src/stdlib/json.rs -- --test std_003_json`
- **Expected runtime**: ~7-10 minutes
- **Mutants**: 25 found
- **Status**: Waiting for lock (runs after http)

## Toyota Way Principles Applied

1. **Jidoka (Stop the Line)**:
   - Discovered timeout issue → STOPPED immediately
   - Did NOT proceed with incomplete testing
   - Root cause analysis before continuing

2. **Genchi Genbutsu (Go and See)**:
   - Empirical data: 95s build + 300s test > 300s timeout
   - Investigated cargo mutants documentation
   - Tested FAST approach: 101s build + 0.3s test = SUCCESS

3. **Kaizen (Continuous Improvement)**:
   - Problem: ALL tests approach = timeout
   - Improvement: Targeted testing = 90%+ faster
   - Documentation: Prevent future teams from same mistake

## Key Learnings

1. **Mutation testing requires targeted approach** for large codebases
2. **Integration tests are sufficient** for stdlib thin wrapper validation
3. **~20 focused tests >> 3662 unfocused tests** for mutation coverage
4. **Configuration is critical** - wrong config = hours wasted

## Next Steps

1. ✅ STD-001 (fs): COMPLETE - 100% mutation coverage
2. ⏳ STD-002 (http): Running FAST mutation tests
3. ⏳ STD-003 (json): Queued for FAST mutation tests
4. 📋 STD-004 (path): Ready for FAST mutation tests after 001/002/003 complete
5. 📊 Update roadmap.yaml with final results
6. 🎯 Change phase status to COMPLETE once all ≥75%

## Sustainability

This FAST approach is sustainable for:
- **Developer workflow**: 7-10 minutes per module (acceptable)
- **CI/CD integration**: Fast enough for pre-merge validation
- **Iterative development**: Quick feedback loop for test enhancement

**Recommendation**: Use FAST mutation testing (`-- --test MODULE`) as standard practice for all stdlib modules.
