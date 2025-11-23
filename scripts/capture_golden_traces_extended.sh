#!/bin/bash
# Extended Golden Trace Capture Script for ruchy
#
# Captures syscall traces for ruchy examples using Renacer.
# Includes: basics, control flow, algorithms, dataframes, async/await, file I/O
#
# Usage: ./scripts/capture_golden_traces_extended.sh

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

echo -e "${BLUE}=== Capturing Extended Golden Traces for ruchy ===${NC}"
echo -e "Binary: $RUCHY_BIN"
echo -e "Output: $TRACES_DIR/"
echo ""

# ==============================================================================
# Trace 1: basics (basic language features)
# ==============================================================================
echo -e "${GREEN}[1/6]${NC} Capturing: basics (01_basics.ruchy)"

renacer --format json -- "$RUCHY_BIN" run examples/01_basics.ruchy 2>&1 | \
    grep -v "^=== \|^Integer\|^Float\|^String\|^Boolean\|^Hello\|^Uppercase\|^Lowercase\|^Length\|^42\|^String '\|^Int to string\|^Type of" | \
    head -1 > "$TRACES_DIR/basics.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/basics.json"

renacer --summary --timing -- "$RUCHY_BIN" run examples/01_basics.ruchy 2>&1 | \
    tail -n +2 > "$TRACES_DIR/basics_summary.txt"

# ==============================================================================
# Trace 2: control_flow (if/else, loops, match)
# ==============================================================================
echo -e "${GREEN}[2/6]${NC} Capturing: control_flow (03_control_flow.ruchy)"

renacer --format json -- "$RUCHY_BIN" run examples/03_control_flow.ruchy 2>&1 | \
    grep -v "^=== \|^is\|^Counter\|^Sum\|^Found\|^Fizz\|^Buzz\|^Number\|^Score\|^Day\|^Result" | \
    head -1 > "$TRACES_DIR/control_flow.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/control_flow.json"

renacer --summary --timing -- "$RUCHY_BIN" run examples/03_control_flow.ruchy 2>&1 | \
    tail -n +2 > "$TRACES_DIR/control_flow_summary.txt"

# ==============================================================================
# Trace 3: algorithms (computational examples)
# ==============================================================================
echo -e "${GREEN}[3/6]${NC} Capturing: algorithms (18_algorithms.ruchy)"

renacer --format json -- "$RUCHY_BIN" run examples/18_algorithms.ruchy 2>&1 | \
    grep -v "^=== \|^Factorial\|^Fibonacci\|^Prime\|^Bubble\|^Binary\|^Sorted\|^Found\|^GCD\|^LCM\|^Palindrome" | \
    head -1 > "$TRACES_DIR/algorithms.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/algorithms.json"

renacer --summary --timing -- "$RUCHY_BIN" run examples/18_algorithms.ruchy 2>&1 | \
    tail -n +2 > "$TRACES_DIR/algorithms_summary.txt"

# ==============================================================================
# Trace 4: dataframes (DataFrame operations) - SPRINT 2
# ==============================================================================
echo -e "${GREEN}[4/6]${NC} Capturing: dataframes (08_dataframes.ruchy)"

if [ -f examples/08_dataframes.ruchy ]; then
    renacer --format json -- "$RUCHY_BIN" run examples/08_dataframes.ruchy 2>&1 | \
        grep -v "^=== \|^DataFrame\|^Column\|^Row\|^Filter\|^Map\|^Group" | \
        head -1 > "$TRACES_DIR/dataframes.json" 2>/dev/null || \
        echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/dataframes.json"

    renacer --summary --timing -- "$RUCHY_BIN" run examples/08_dataframes.ruchy 2>&1 | \
        tail -n +2 > "$TRACES_DIR/dataframes_summary.txt"
else
    echo -e "${YELLOW}  Skipping: 08_dataframes.ruchy not found${NC}"
fi

# ==============================================================================
# Trace 5: async_await (async/await patterns) - SPRINT 2
# ==============================================================================
echo -e "${GREEN}[5/6]${NC} Capturing: async_await (09_async_await.ruchy)"

if [ -f examples/09_async_await.ruchy ]; then
    renacer --format json -- "$RUCHY_BIN" run examples/09_async_await.ruchy 2>&1 | \
        grep -v "^=== \|^Async\|^Await\|^Future\|^Task" | \
        head -1 > "$TRACES_DIR/async_await.json" 2>/dev/null || \
        echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/async_await.json"

    renacer --summary --timing -- "$RUCHY_BIN" run examples/09_async_await.ruchy 2>&1 | \
        tail -n +2 > "$TRACES_DIR/async_await_summary.txt"
else
    echo -e "${YELLOW}  Skipping: 09_async_await.ruchy not found${NC}"
fi

# ==============================================================================
# Trace 6: file_io (File I/O operations) - SPRINT 2
# ==============================================================================
echo -e "${GREEN}[6/6]${NC} Capturing: file_io (11_file_io.ruchy)"

if [ -f examples/11_file_io.ruchy ]; then
    renacer --format json -- "$RUCHY_BIN" run examples/11_file_io.ruchy 2>&1 | \
        grep -v "^=== \|^File\|^Read\|^Write\|^Directory" | \
        head -1 > "$TRACES_DIR/file_io.json" 2>/dev/null || \
        echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/file_io.json"

    renacer --summary --timing -- "$RUCHY_BIN" run examples/11_file_io.ruchy 2>&1 | \
        tail -n +2 > "$TRACES_DIR/file_io_summary.txt"
else
    echo -e "${YELLOW}  Skipping: 11_file_io.ruchy not found${NC}"
fi

# ==============================================================================
# Completion
# ==============================================================================
echo ""
echo -e "${GREEN}=== Golden Trace Capture Complete ===${NC}"
echo ""
echo "Files generated:"
ls -lh "$TRACES_DIR" | tail -n +2 | awk '{printf "  %s (%s)\n", $9, $5}'
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Review traces: cat golden_traces/*_summary.txt"
echo "  2. Validate budgets: make golden-traces-validate"
echo "  3. Commit traces: git add golden_traces/"
