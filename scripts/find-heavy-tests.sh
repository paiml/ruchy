#!/bin/bash
# find-heavy-tests.sh - Identify memory-intensive tests

echo "Finding heavy tests..."
echo "Test Name | Memory Usage (KB)"
echo "--------------------------------"

# Create temp file for results
RESULTS_FILE=$(mktemp)

# Run each test individually and measure memory
for test in $(cargo test -- --list 2>/dev/null | grep "test::" | cut -d: -f2 | head -20); do
    echo -n "Testing: $test... "
    
    # Run test with memory measurement
    MEMORY=$(/usr/bin/time -f "%M" cargo test $test -- --exact --nocapture 2>&1 | tail -1)
    
    if [[ "$MEMORY" =~ ^[0-9]+$ ]]; then
        echo "$MEMORY KB"
        echo "$MEMORY $test" >> "$RESULTS_FILE"
    else
        echo "skipped"
    fi
done

echo ""
echo "Top 10 Memory-Intensive Tests:"
echo "==============================="
sort -rn "$RESULTS_FILE" | head -10 | while read mem test; do
    printf "%-40s %10s KB\n" "$test" "$mem"
done

# Cleanup
rm -f "$RESULTS_FILE"