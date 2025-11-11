#!/usr/bin/env bash
# Mutation testing for class-related code paths
#
# TARGET: ≥75% CAUGHT/MISSED ratio for class code
# FILES:
# - src/frontend/parser/expressions_helpers/classes.rs (897 lines)
# - src/frontend/parser/expressions_helpers/structs.rs (370 lines)
# - src/frontend/parser/expressions_helpers/impls.rs (141 lines)
# - src/runtime/interpreter.rs (class methods only)
#
# STRATEGY: File-by-file mutation testing (incremental approach)
# Each file takes 5-30 minutes depending on complexity
#
# Usage: bash .pmat/run_class_mutations.sh [file_name]

set -euo pipefail

TIMEOUT=300  # 5 minutes per file
OUTPUT_DIR="target/mutations"
mkdir -p "$OUTPUT_DIR"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

run_mutation_test() {
    local file=$1
    local basename=$(basename "$file" .rs)
    local output_file="$OUTPUT_DIR/${basename}_mutations.txt"

    log_info "Running mutation tests for $file"
    log_info "Timeout: ${TIMEOUT}s per mutation"
    log_info "Output: $output_file"

    cargo mutants --file "$file" --timeout "$TIMEOUT" \
        --output json \
        --json-file "$OUTPUT_DIR/${basename}_mutations.json" \
        | tee "$output_file"

    # Analyze results
    local caught=$(grep -c "CAUGHT" "$output_file" || echo "0")
    local missed=$(grep -c "MISSED" "$output_file" || echo "0")
    local total=$((caught + missed))

    if [ "$total" -gt 0 ]; then
        local ratio=$(echo "scale=2; $caught * 100 / $total" | bc)
        log_info "Results: $caught CAUGHT, $missed MISSED, ${ratio}% coverage"

        if (( $(echo "$ratio >= 75" | bc -l) )); then
            log_info "✅ PASSED: Mutation coverage ≥75%"
        else
            log_warn "⚠️  BELOW TARGET: Mutation coverage <75%"
            log_warn "Add tests targeting MISSED mutations"
        fi
    else
        log_warn "No mutations found or all timed out"
    fi
}

# Main execution
if [ $# -eq 1 ]; then
    # Run specific file
    run_mutation_test "$1"
else
    # Run all class-related files
    log_info "Running mutation tests for all class-related files"
    log_info "This will take approximately 30-60 minutes total"

    run_mutation_test "src/frontend/parser/expressions_helpers/classes.rs"
    run_mutation_test "src/frontend/parser/expressions_helpers/structs.rs"
    run_mutation_test "src/frontend/parser/expressions_helpers/impls.rs"

    # For interpreter.rs, we'd need to target specific functions
    # This would take too long for the full file, so we skip it for now
    # log_warn "Skipping interpreter.rs (too large for full mutation testing)"
    # log_info "Consider targeted mutation testing for specific functions"

    log_info "All mutation tests complete"
    log_info "Results saved in $OUTPUT_DIR/"
fi
