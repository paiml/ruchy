#!/bin/bash
# Add tests to modules with lowest test coverage
# Focuses on high-impact, low-effort test additions

echo "=== Targeted Test Addition Plan ==="

# Priority 1: Add basic tests to completely untested small files
echo "Priority 1: Small untested files (<500 lines):"
for file in $(find src -name '*.rs' -type f ! -exec grep -l '#\[test\]' {} \; 2>/dev/null); do
    lines=$(wc -l < "$file")
    if [ "$lines" -lt 500 ]; then
        echo "  $file ($lines lines) - needs basic test module"
    fi
done | head -10

echo ""
echo "Priority 2: Add tests to critical modules:"
echo "  src/api_docs.rs - API documentation (needs doc tests)"
echo "  src/error_recovery_enhanced.rs - Error recovery (needs unit tests)"
echo "  src/performance_optimizations.rs - Performance (needs benchmarks)"

echo ""
echo "Generating test template for highest-impact module..."