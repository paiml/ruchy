#!/bin/bash
# Overnight mutation testing for Sprint 9 Phase 3 remaining files
# Created: 2025-10-06
# Purpose: Run complete mutation tests on 7 large runtime files (estimated 10-15 hours)

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_DIR="/home/noah/src/ruchy/.pmat/mutation_logs"
mkdir -p "$LOG_DIR"

echo "🌙 Starting overnight mutation testing at $(date)"
echo "📂 Logs will be saved to: $LOG_DIR"
echo ""

# Array of files to test
FILES=(
    "src/runtime/eval_pattern.rs"
    "src/runtime/cache.rs"
    "src/runtime/eval_loops.rs"
    "src/runtime/eval_method_dispatch.rs"
    "src/runtime/safe_arena.rs"
    "src/runtime/eval_string.rs"
    "src/runtime/inspect.rs"
)

# Test each file sequentially
for FILE in "${FILES[@]}"; do
    BASENAME=$(basename "$FILE" .rs)
    OUTPUT_FILE="$LOG_DIR/${BASENAME}_mutations_${TIMESTAMP}.txt"

    echo "🧪 Testing: $FILE"
    echo "   Output: $OUTPUT_FILE"
    echo "   Started: $(date)"

    # Run mutation test with 600s timeout per mutant, no timing overhead
    cargo mutants --file "$FILE" --timeout 600 --no-times 2>&1 | tee "$OUTPUT_FILE"

    echo "   Completed: $(date)"
    echo ""

    # Extract summary for quick review
    echo "📊 Summary for $BASENAME:"
    grep -E "(Found|mutants tested|CAUGHT|MISSED)" "$OUTPUT_FILE" | tail -5
    echo ""
done

echo "🎉 All mutation tests completed at $(date)"
echo ""
echo "📈 Final Summary:"
echo "================="
for FILE in "${FILES[@]}"; do
    BASENAME=$(basename "$FILE" .rs)
    OUTPUT_FILE="$LOG_DIR/${BASENAME}_mutations_${TIMESTAMP}.txt"

    echo ""
    echo "$BASENAME:"
    grep -E "(Found|mutants tested)" "$OUTPUT_FILE" | tail -2
done

echo ""
echo "✅ Overnight mutation testing complete!"
echo "📂 All logs saved to: $LOG_DIR"
