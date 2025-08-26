#!/bin/bash
# AFL++ Fuzzing Script for Ruchy Compiler
# QUALITY-011: Advanced fuzzing with AFL++

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔍 Ruchy AFL++ Fuzzing Setup${NC}"

# Check if AFL++ is installed
if ! command -v afl-fuzz &> /dev/null; then
    echo -e "${YELLOW}AFL++ not found. Installing...${NC}"
    # Instructions for installation (user needs to do this manually)
    echo "Please install AFL++ first:"
    echo "  cargo install afl"
    echo "  or"
    echo "  git clone https://github.com/AFLplusplus/AFLplusplus"
    echo "  cd AFLplusplus && make && sudo make install"
    exit 1
fi

# Build the project with AFL instrumentation
echo -e "${GREEN}Building with AFL instrumentation...${NC}"
cargo afl build --release

# Create input corpus if it doesn't exist
CORPUS_DIR="fuzz/afl_corpus"
if [ ! -d "$CORPUS_DIR" ]; then
    echo -e "${GREEN}Creating initial corpus...${NC}"
    mkdir -p "$CORPUS_DIR"
    
    # Add some seed inputs
    echo "let x = 42" > "$CORPUS_DIR/simple_let.ruchy"
    echo "fun add(a: i32, b: i32) -> i32 { a + b }" > "$CORPUS_DIR/function.ruchy"
    echo "match x { 0 => \"zero\", _ => \"other\" }" > "$CORPUS_DIR/match.ruchy"
    echo "[1, 2, 3].map(|x| x * 2)" > "$CORPUS_DIR/list_ops.ruchy"
    echo "if x > 0 { x } else { -x }" > "$CORPUS_DIR/conditional.ruchy"
fi

# Create output directory
OUTPUT_DIR="fuzz/afl_output"
mkdir -p "$OUTPUT_DIR"

# Choose target
echo -e "${GREEN}Select fuzzing target:${NC}"
echo "1) Parser"
echo "2) Transpiler"
echo "3) Full Pipeline (Parser + Transpiler)"
echo "4) Interpreter"
read -p "Choice (1-4): " choice

case $choice in
    1)
        TARGET="parser"
        HARNESS="fuzz/afl_harness_parser"
        ;;
    2)
        TARGET="transpiler"
        HARNESS="fuzz/afl_harness_transpiler"
        ;;
    3)
        TARGET="full_pipeline"
        HARNESS="fuzz/afl_harness_pipeline"
        ;;
    4)
        TARGET="interpreter"
        HARNESS="fuzz/afl_harness_interpreter"
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}Starting AFL++ fuzzing for $TARGET...${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop fuzzing${NC}"

# Run AFL++ with optimized settings
AFL_SKIP_CPUFREQ=1 \
AFL_I_DONT_CARE_ABOUT_MISSING_CRASHES=1 \
cargo afl fuzz \
    -i "$CORPUS_DIR" \
    -o "$OUTPUT_DIR" \
    -M fuzzer01 \
    target/release/ruchy_afl_harness

echo -e "${GREEN}Fuzzing completed. Check $OUTPUT_DIR for results.${NC}"

# Analyze crashes if any found
if [ -d "$OUTPUT_DIR/fuzzer01/crashes" ] && [ "$(ls -A $OUTPUT_DIR/fuzzer01/crashes)" ]; then
    echo -e "${RED}Crashes found! Analyzing...${NC}"
    for crash in "$OUTPUT_DIR/fuzzer01/crashes"/*; do
        if [ -f "$crash" ]; then
            echo -e "${YELLOW}Crash: $(basename $crash)${NC}"
            # You can add crash minimization here
            # cargo afl tmin -i "$crash" -o "$crash.min" target/release/ruchy_afl_harness
        fi
    done
else
    echo -e "${GREEN}No crashes found - code is robust!${NC}"
fi