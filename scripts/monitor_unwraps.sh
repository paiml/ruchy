#!/bin/bash
# Monitor unwrap() usage and prevent regression
# This script tracks unwrap() calls in production code and ensures they don't increase

set -e

BASELINE_FILE=".unwrap_baseline"
THRESHOLD=314  # Current baseline after cleanup

echo "=== Unwrap() Usage Monitor ==="
echo

# Count production unwraps (excluding tests)
count_production_unwraps() {
    local count=0
    for file in $(find src -name "*.rs" -not -path "*/tests/*" -not -name "*_test.rs"); do
        # Count unwraps that aren't in doc comments or test functions
        local file_count=$(grep '\.unwrap()' "$file" 2>/dev/null | \
            grep -v '///' | \
            grep -v '#\[test\]' | \
            grep -v '#\[cfg(test)\]' | \
            grep -v 'fn test_' | \
            wc -l)
        count=$((count + file_count))
    done
    echo $count
}

# Get current count
CURRENT_COUNT=$(count_production_unwraps)

echo "Current production unwrap() count: $CURRENT_COUNT"
echo "Baseline threshold: $THRESHOLD"
echo

# Check if we have a baseline file
if [ -f "$BASELINE_FILE" ]; then
    BASELINE=$(cat "$BASELINE_FILE")
    echo "Previous baseline: $BASELINE"
    
    if [ "$CURRENT_COUNT" -gt "$BASELINE" ]; then
        echo "❌ ERROR: unwrap() count increased from $BASELINE to $CURRENT_COUNT"
        echo "New unwraps detected! Please use proper error handling instead:"
        echo "  - Use ? operator for propagating errors"
        echo "  - Use .expect() with descriptive messages"
        echo "  - Use .unwrap_or() / .unwrap_or_else() for defaults"
        echo "  - Use .context() from anyhow for better error messages"
        exit 1
    elif [ "$CURRENT_COUNT" -lt "$BASELINE" ]; then
        echo "✅ Great! unwrap() count decreased from $BASELINE to $CURRENT_COUNT"
        echo "Updating baseline..."
        echo "$CURRENT_COUNT" > "$BASELINE_FILE"
    else
        echo "✅ unwrap() count unchanged at $BASELINE"
    fi
else
    echo "Creating initial baseline at $CURRENT_COUNT unwraps"
    echo "$CURRENT_COUNT" > "$BASELINE_FILE"
fi

# Report top files with unwraps for attention
echo
echo "=== Top files with production unwraps (for future cleanup) ==="
for file in $(find src -name "*.rs" -not -path "*/tests/*" -not -name "*_test.rs"); do
    count=$(grep -c '\.unwrap()' "$file" 2>/dev/null || echo 0)
    if [ "$count" -gt "0" ]; then
        real_count=$(grep '\.unwrap()' "$file" | grep -v '///' | grep -v '#\[test\]' | wc -l)
        if [ "$real_count" -gt "0" ]; then
            echo "$real_count $file"
        fi
    fi
done | sort -rn | head -5

echo
echo "=== Unwrap categories breakdown ==="
echo "SystemTime unwraps: $(grep -r 'SystemTime.*unwrap()' src --include='*.rs' 2>/dev/null | wc -l)"
echo "Parse unwraps: $(grep -r 'parse().*unwrap()' src --include='*.rs' 2>/dev/null | wc -l)"
echo "Lock unwraps: $(grep -r 'lock().*expect(' src --include='*.rs' 2>/dev/null | wc -l) (now using expect)"
echo "Join unwraps: $(grep -r 'join().*unwrap()' src --include='*.rs' 2>/dev/null | wc -l)"

exit 0