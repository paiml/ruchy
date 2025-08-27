#!/bin/bash
# Quick Quality Tools Performance Benchmark

echo "🚀 Ruchy Quality Tools Performance Benchmark"
echo "==========================================="
echo ""

# Create test file
TEST_FILE="/tmp/benchmark_test.ruchy"
cat > "$TEST_FILE" << 'EOF'
fn fibonacci(n) {
    if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
}

fn main() {
    println("Fib(10) = " + fibonacci(10).to_string());
}

main();
EOF

echo "📊 Benchmarking each quality tool..."
echo ""

# Benchmark ruchy test
echo "1. ruchy test (on tests/ directory):"
START=$(date +%s%N)
ruchy test tests/ 2>&1 | grep -E "(passed|Duration)" || true
END=$(date +%s%N)
ELAPSED=$((($END - $START) / 1000000))
echo "   Execution time: ${ELAPSED}ms"
echo ""

# Benchmark ruchy lint  
echo "2. ruchy lint (on single file):"
START=$(date +%s%N)
ruchy lint "$TEST_FILE" 2>&1 | grep -E "(issues|No issues)" || true
END=$(date +%s%N)
ELAPSED=$((($END - $START) / 1000000))
echo "   Execution time: ${ELAPSED}ms"
echo ""

# Benchmark ruchy score
echo "3. ruchy score (on single file):"
START=$(date +%s%N)
ruchy score "$TEST_FILE" 2>&1 | grep "Score:" || true
END=$(date +%s%N)
ELAPSED=$((($END - $START) / 1000000))
echo "   Execution time: ${ELAPSED}ms"
echo ""

# Benchmark ruchy prove
echo "4. ruchy prove (on single file):"
START=$(date +%s%N)
ruchy prove "$TEST_FILE" 2>&1 | head -2 || true
END=$(date +%s%N)
ELAPSED=$((($END - $START) / 1000000))
echo "   Execution time: ${ELAPSED}ms"
echo ""

# Benchmark with larger test suite
echo "5. ruchy test on multiple files:"
mkdir -p /tmp/test_suite
for i in {1..10}; do
    cat > "/tmp/test_suite/test_$i.ruchy" << EOF
fn test_$i() {
    assert(1 + 1 == 2, "Math works");
}
test_$i();
EOF
done

START=$(date +%s%N)
ruchy test /tmp/test_suite/ 2>&1 | grep -E "(Total|Duration)" || true
END=$(date +%s%N)
ELAPSED=$((($END - $START) / 1000000))
echo "   Execution time: ${ELAPSED}ms"
echo ""

# Cleanup
rm -rf /tmp/test_suite

echo "✅ Benchmark complete!"