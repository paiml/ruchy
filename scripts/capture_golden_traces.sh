#!/bin/bash
# Golden Trace Capture Script for ruchy
#
# Captures syscall traces for ruchy (systems scripting language) examples using Renacer.
# Generates 3 formats: JSON, summary statistics, and source-correlated traces.
#
# Usage: ./scripts/capture_golden_traces.sh

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
TRACES_DIR="golden_traces"
RUCHY_BIN="./target/release/ruchy"

# Ensure renacer is installed
if ! command -v renacer &> /dev/null; then
    echo -e "${YELLOW}Renacer not found. Installing from crates.io...${NC}"
    cargo install renacer --version 0.6.2
fi

# Build ruchy binary
echo -e "${YELLOW}Building release ruchy binary...${NC}"
cargo build --release --bin ruchy

# Create traces directory
mkdir -p "$TRACES_DIR"

echo -e "${BLUE}=== Capturing Golden Traces for ruchy ===${NC}"
echo -e "Binary: $RUCHY_BIN"
echo -e "Output: $TRACES_DIR/"
echo ""

# ==============================================================================
# Trace 1: basics (basic language features)
# ==============================================================================
echo -e "${GREEN}[1/3]${NC} Capturing: basics (01_basics.ruchy)"

renacer --format json -- "$RUCHY_BIN" run examples/01_basics.ruchy 2>&1 | \
    grep -v "^=== \\|^Integer\\|^Float\\|^String\\|^Boolean\\|^Hello\\|^Uppercase\\|^Lowercase\\|^Length\\|^42\\|^String '\\|^Int to string\\|^Type of" | \
    head -1 > "$TRACES_DIR/basics.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/basics.json"

renacer --summary --timing -- "$RUCHY_BIN" run examples/01_basics.ruchy 2>&1 | \
    tail -n +2 > "$TRACES_DIR/basics_summary.txt"

renacer -s --format json -- "$RUCHY_BIN" run examples/01_basics.ruchy 2>&1 | \
    grep -v "^=== \\|^Integer\\|^Float\\|^String\\|^Boolean\\|^Hello\\|^Uppercase\\|^Lowercase\\|^Length\\|^42\\|^String '\\|^Int to string\\|^Type of" | \
    head -1 > "$TRACES_DIR/basics_source.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/basics_source.json"

# ==============================================================================
# Trace 2: control_flow (if/else, loops, match)
# ==============================================================================
echo -e "${GREEN}[2/3]${NC} Capturing: control_flow (03_control_flow.ruchy)"

renacer --format json -- "$RUCHY_BIN" run examples/03_control_flow.ruchy 2>&1 | \
    grep -v "^=== \\|^is\\|^Counter\\|^Sum\\|^Found\\|^Fizz\\|^Buzz\\|^Number\\|^Score\\|^Day\\|^Result" | \
    head -1 > "$TRACES_DIR/control_flow.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/control_flow.json"

renacer --summary --timing -- "$RUCHY_BIN" run examples/03_control_flow.ruchy 2>&1 | \
    tail -n +2 > "$TRACES_DIR/control_flow_summary.txt"

# ==============================================================================
# Trace 3: algorithms (computational examples)
# ==============================================================================
echo -e "${GREEN}[3/3]${NC} Capturing: algorithms (18_algorithms.ruchy)"

renacer --format json -- "$RUCHY_BIN" run examples/18_algorithms.ruchy 2>&1 | \
    grep -v "^=== \\|^Factorial\\|^Fibonacci\\|^Prime\\|^Bubble\\|^Binary\\|^Sorted\\|^Found\\|^GCD\\|^LCM\\|^Palindrome" | \
    head -1 > "$TRACES_DIR/algorithms.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/algorithms.json"

renacer --summary --timing -- "$RUCHY_BIN" run examples/18_algorithms.ruchy 2>&1 | \
    tail -n +2 > "$TRACES_DIR/algorithms_summary.txt"

# ==============================================================================
# Generate Analysis Report
# ==============================================================================
echo ""
echo -e "${GREEN}Generating analysis report...${NC}"

cat > "$TRACES_DIR/ANALYSIS.md" << 'EOF'
# Golden Trace Analysis Report - ruchy

## Overview

This directory contains golden traces captured from ruchy (systems scripting language that transpiles to Rust) examples.

## Trace Files

| File | Description | Format |
|------|-------------|--------|
| `basics.json` | Basic language features (variables, types, arithmetic) | JSON |
| `basics_summary.txt` | Basics syscall summary | Text |
| `basics_source.json` | Basics with source locations | JSON |
| `control_flow.json` | Control flow (if/else, loops, match) | JSON |
| `control_flow_summary.txt` | Control flow syscall summary | Text |
| `algorithms.json` | Algorithm implementations (factorial, fibonacci, sorting) | JSON |
| `algorithms_summary.txt` | Algorithms syscall summary | Text |

## How to Use These Traces

### 1. Regression Testing

Compare new builds against golden traces:

```bash
# Capture new trace
renacer --format json -- ./target/release/ruchy run examples/01_basics.ruchy > new_trace.json

# Compare with golden
diff golden_traces/basics.json new_trace.json

# Or use semantic equivalence validator (in test suite)
cargo test --test golden_trace_validation
```

### 2. Performance Budgeting

Check if new build meets performance requirements:

```bash
# Run with assertions
cargo test --test performance_assertions

# Or manually check against summary
cat golden_traces/basics_summary.txt
```

### 3. CI/CD Integration

Add to `.github/workflows/ci.yml`:

```yaml
- name: Validate ruchy Performance
  run: |
    renacer --format json -- ./target/release/ruchy run examples/01_basics.ruchy > trace.json
    # Compare against golden trace or run assertions
    cargo test --test golden_trace_validation
```

## Trace Interpretation Guide

### JSON Trace Format

```json
{
  "version": "0.6.2",
  "format": "renacer-json-v1",
  "syscalls": [
    {
      "name": "write",
      "args": [["fd", "1"], ["buf", "Results: [...]"], ["count", "25"]],
      "result": 25
    }
  ]
}
```

### Summary Statistics Format

```
% time     seconds  usecs/call     calls    errors syscall
------ ----------- ----------- --------- --------- ----------------
 19.27    0.000137          10        13           mmap
 14.35    0.000102          17         6           write
...
```

**Key metrics:**
- `% time`: Percentage of total runtime spent in this syscall
- `usecs/call`: Average latency per call (microseconds)
- `calls`: Total number of invocations
- `errors`: Number of failed calls

## Baseline Performance Metrics

From initial golden trace capture:

| Operation | Runtime | Syscalls | Notes |
|-----------|---------|----------|-------|
| `basics` | TBD | TBD | Basic language features |
| `control_flow` | TBD | TBD | Control flow constructs |
| `algorithms` | TBD | TBD | Algorithm implementations |

## Transpiler Performance Characteristics

### Expected Syscall Patterns

**Script Parsing**:
- File I/O for reading .ruchy source files
- Memory allocation for AST construction

**Transpilation**:
- CPU-intensive parsing and code generation
- Minimal syscalls during compilation

**Execution** (interpreted mode):
- Write syscalls for output
- File I/O if script uses file operations
- Memory allocation for runtime state

**Code Generation** (compiled mode):
- Additional file I/O for Rust code output
- Temporary file creation
- Cargo invocation for compilation

### Anti-Pattern Detection

Renacer can detect:

1. **Tight Loop**:
   - Symptom: Excessive loop iterations without I/O
   - Solution: Optimize algorithm or batch operations

2. **God Process**:
   - Symptom: Single process doing too much
   - Solution: Separate parsing from execution

## Next Steps

1. **Set performance baselines** using these golden traces
2. **Add assertions** in `renacer.toml` for automated checking
3. **Integrate with CI** to prevent regressions
4. **Compare interpreted vs compiled** execution traces
5. **Monitor transpilation time** for large scripts

Generated: $(date)
Renacer Version: 0.6.2
ruchy Version: 3.213.0
EOF

# ==============================================================================
# Summary
# ==============================================================================
echo ""
echo -e "${BLUE}=== Golden Trace Capture Complete ===${NC}"
echo ""
echo "Traces saved to: $TRACES_DIR/"
echo ""
echo "Files generated:"
ls -lh "$TRACES_DIR"/*.json "$TRACES_DIR"/*.txt 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo -e "${GREEN}Next steps:${NC}"
echo "  1. Review traces: cat golden_traces/basics_summary.txt"
echo "  2. View JSON: jq . golden_traces/basics.json | less"
echo "  3. Run tests: cargo test --test golden_trace_validation"
echo "  4. Update baselines in ANALYSIS.md with actual metrics"
