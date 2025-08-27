#!/bin/bash
# Quality Tools Performance Benchmark Suite
# Measures execution time and memory usage for all quality tools

set -e

# Configuration
BENCHMARK_DIR="/home/noah/src/ruchy/benchmarks"
RESULTS_DIR="$BENCHMARK_DIR/results"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULTS_FILE="$RESULTS_DIR/benchmark-$TIMESTAMP.json"

# Test files of different sizes
SMALL_FILE="/home/noah/src/ruchy/examples/hello.ruchy"
MEDIUM_FILE="/home/noah/src/ruchy/src/interpreter/mod.rs"  
LARGE_FILE="/home/noah/src/ruchy/src/transpiler/mod.rs"

# Create directories
mkdir -p "$RESULTS_DIR"

# JSON output helper
json_results='{"timestamp":"'$TIMESTAMP'","benchmarks":{}}'

# Function to benchmark a command
benchmark_tool() {
    local tool=$1
    local file=$2
    local size=$3
    
    echo "Benchmarking: ruchy $tool on $size file..."
    
    # Run 5 times and average
    local total_time=0
    local runs=5
    
    for i in $(seq 1 $runs); do
        # Use GNU time for detailed metrics
        local output=$(/usr/bin/time -f "%e %M" -o /tmp/bench_time.txt ruchy $tool "$file" 2>&1 > /dev/null || true)
        
        if [ -f /tmp/bench_time.txt ]; then
            local elapsed=$(awk '{print $1}' /tmp/bench_time.txt)
            local memory=$(awk '{print $2}' /tmp/bench_time.txt)
            total_time=$(echo "$total_time + $elapsed" | bc)
        fi
    done
    
    local avg_time=$(echo "scale=3; $total_time / $runs" | bc)
    echo "  Average time: ${avg_time}s"
    
    # Update JSON
    json_results=$(echo "$json_results" | jq ".benchmarks.\"${tool}_${size}\" = {\"avg_time\": $avg_time, \"runs\": $runs}")
}

# Create test files if needed
if [ ! -f "$SMALL_FILE" ]; then
    echo 'println("Hello, World!");' > "$SMALL_FILE"
fi

echo "🚀 Ruchy Quality Tools Performance Benchmark"
echo "==========================================="
echo "Timestamp: $TIMESTAMP"
echo ""

# Benchmark each tool with each file size
for tool in test lint score prove; do
    echo "📊 Benchmarking: ruchy $tool"
    echo "----------------------------"
    
    # Small file
    if [ -f "$SMALL_FILE" ]; then
        benchmark_tool "$tool" "$SMALL_FILE" "small"
    fi
    
    # For test, use directory
    if [ "$tool" = "test" ]; then
        benchmark_tool "test" "/home/noah/src/ruchy/tests" "directory"
    fi
    
    echo ""
done

# Special benchmarks for specific features
echo "📊 Special Benchmarks"
echo "--------------------"

# Lint with auto-fix
echo "Benchmarking: ruchy lint --fix..."
time_start=$(date +%s.%N)
ruchy lint "$SMALL_FILE" --fix 2>&1 > /dev/null || true
time_end=$(date +%s.%N)
elapsed=$(echo "$time_end - $time_start" | bc)
echo "  Time: ${elapsed}s"
json_results=$(echo "$json_results" | jq ".benchmarks.\"lint_autofix\" = {\"time\": $elapsed}")

# Score with deep analysis
echo "Benchmarking: ruchy score --deep..."
time_start=$(date +%s.%N)
ruchy score "$SMALL_FILE" --deep 2>&1 > /dev/null || true
time_end=$(date +%s.%N)
elapsed=$(echo "$time_end - $time_start" | bc)
echo "  Time: ${elapsed}s"
json_results=$(echo "$json_results" | jq ".benchmarks.\"score_deep\" = {\"time\": $elapsed}")

# Save results
echo "$json_results" | jq '.' > "$RESULTS_FILE"

echo ""
echo "✅ Benchmark complete!"
echo "Results saved to: $RESULTS_FILE"
echo ""
echo "Summary:"
echo "--------"
echo "$json_results" | jq '.benchmarks | to_entries | .[] | "\(.key): \(.value.avg_time // .value.time)s"' -r | sort