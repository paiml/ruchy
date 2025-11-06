#!/usr/bin/env bash
# run_type_inference_mutations.sh - Run mutation testing on type inference code
# TRANSPILER-TYPE-INFER-PARAMS + TRANSPILER-TYPE-INFER-EXPR validation
#
# Usage:
#   ./run_type_inference_mutations.sh          # Run with defaults
#   ./run_type_inference_mutations.sh quick    # 30s timeout per mutant
#   ./run_type_inference_mutations.sh full     # 60s timeout per mutant

set -euo pipefail

# Configuration
TARGET_FILE="src/backend/transpiler/statements.rs"
OUTPUT_FILE="/tmp/type_inference_mutations_$(date +%Y%m%d_%H%M%S).txt"
TIMEOUT="${1:-60}"

case "${1:-full}" in
    quick)
        TIMEOUT=30
        echo "🏃 Quick mode: 30s timeout per mutant"
        ;;
    full)
        TIMEOUT=60
        echo "🚀 Full mode: 60s timeout per mutant"
        ;;
    *)
        TIMEOUT="${1}"
        echo "⚙️  Custom mode: ${TIMEOUT}s timeout per mutant"
        ;;
esac

echo "=================================================="
echo "Mutation Testing: Type Inference Code"
echo "=================================================="
echo "Target: ${TARGET_FILE}"
echo "Output: ${OUTPUT_FILE}"
echo "Timeout: ${TIMEOUT}s per mutant"
echo "Expected Runtime: 30-60 minutes"
echo ""
echo "Target Methods (110 lines):"
echo "  - infer_return_type_from_params() [Lines 878-917, Complexity 9]"
echo "  - get_final_expression() [Lines 921-931, Complexity 3]"
echo "  - trace_param_assignments() [Lines 935-962, Complexity 6]"
echo "  - infer_expr_type_from_params() [Lines 968-984, Complexity 6]"
echo ""
echo "Test Coverage:"
echo "  - tests/test_transpiler_type_infer_from_params.rs (5 tests)"
echo "  - tests/transpiler_property_comprehensive.rs (35K cases)"
echo ""
echo "Target: ≥75% CAUGHT/MISSED ratio"
echo "=================================================="
echo ""

# Ensure clean state
echo "🧹 Ensuring clean build state..."
cargo build --tests --quiet 2>&1 | tail -5

echo "🧬 Running mutation tests..."
echo "(This will take 30-60 minutes - grab some coffee ☕)"
echo ""

# Run mutation testing
timeout $((TIMEOUT * 100)) cargo mutants \
    --file "${TARGET_FILE}" \
    --timeout "${TIMEOUT}" \
    2>&1 | tee "${OUTPUT_FILE}"

# Extract and display summary
echo ""
echo "=================================================="
echo "Mutation Testing Results"
echo "=================================================="
grep -E "(^Found|^Tested|caught|missed|^CAUGHT|^MISSED|Summary)" "${OUTPUT_FILE}" || true

echo ""
echo "Full results saved to: ${OUTPUT_FILE}"
echo ""

# Calculate caught rate
CAUGHT=$(grep -oP "(?<=CAUGHT: )\d+" "${OUTPUT_FILE}" || echo "0")
MISSED=$(grep -oP "(?<=MISSED: )\d+" "${OUTPUT_FILE}" || echo "0")
TOTAL=$((CAUGHT + MISSED))

if [ "${TOTAL}" -gt 0 ]; then
    RATE=$(awk "BEGIN {printf \"%.1f\", (${CAUGHT}/${TOTAL})*100}")
    echo "Mutation Coverage: ${CAUGHT}/${TOTAL} caught (${RATE}%)"

    if (( $(echo "${RATE} >= 75.0" | bc -l) )); then
        echo "✅ SUCCESS: ≥75% mutation coverage achieved!"
        exit 0
    else
        echo "⚠️  WARNING: <75% mutation coverage (target: ≥75%)"
        echo "   Add more tests to catch missed mutations"
        exit 1
    fi
else
    echo "❌ ERROR: Could not parse mutation results"
    exit 1
fi
