#!/bin/bash
# scripts/critical-path-optimization.sh
# WebAssembly Extreme Quality Assurance Framework v3.0
# Critical Path Optimization Script

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Starting critical path optimization analysis...${NC}"

# Create output directory
mkdir -p target/optimization

echo -e "${YELLOW}Phase 1: Hot path identification${NC}"

# Profile compilation paths
echo -e "${YELLOW}Analyzing compilation critical paths...${NC}"

cat > target/optimization/profile-compilation.sh << 'EOF'
#!/bin/bash
# Profile different compilation scenarios

echo "Testing basic compilation..."
time cargo build --quiet >/dev/null 2>&1 || echo "Build failed"

echo "Testing WASM compilation..."
if command -v wasm-pack &> /dev/null; then
    time wasm-pack build --target web --out-dir target/wasm-profile --quiet >/dev/null 2>&1 || echo "WASM build failed"
else
    echo "wasm-pack not available"
fi

echo "Testing test compilation..."
time cargo test --no-run --quiet >/dev/null 2>&1 || echo "Test compilation failed"

echo "Testing benchmark compilation..."
time cargo bench --no-run --quiet >/dev/null 2>&1 || echo "Benchmark compilation failed"
EOF

chmod +x target/optimization/profile-compilation.sh
./target/optimization/profile-compilation.sh > target/optimization/compilation-times.log 2>&1

echo -e "${YELLOW}Phase 2: Runtime hot path analysis${NC}"

# Create hot path detection script
cat > target/optimization/detect-hotpaths.py << 'EOF'
#!/usr/bin/env python3
"""
Hot path detection for Ruchy compiler critical paths
"""

import os
import re
import subprocess
from pathlib import Path
from collections import defaultdict, Counter

def analyze_source_complexity():
    """Analyze source files for complexity hotspots"""
    hotspots = []

    src_dir = Path("src")
    if not src_dir.exists():
        return hotspots

    for rust_file in src_dir.rglob("*.rs"):
        try:
            with open(rust_file, 'r') as f:
                content = f.read()
                lines = content.split('\n')

                # Count function complexity indicators
                function_lines = defaultdict(int)
                current_function = None
                brace_depth = 0

                for i, line in enumerate(lines):
                    line = line.strip()

                    # Detect function start
                    fn_match = re.match(r'\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)', line)
                    if fn_match:
                        current_function = fn_match.group(1)
                        function_lines[current_function] = 1
                        brace_depth = 0

                    # Count braces for function scope
                    if current_function:
                        brace_depth += line.count('{') - line.count('}')
                        if brace_depth > 0:
                            function_lines[current_function] += 1
                        elif brace_depth == 0 and '}' in line:
                            # Function ended
                            if function_lines[current_function] > 50:
                                hotspots.append({
                                    'file': str(rust_file),
                                    'function': current_function,
                                    'lines': function_lines[current_function],
                                    'type': 'large_function'
                                })
                            current_function = None

                # Detect nested loops (performance concern)
                nested_loops = 0
                for line in lines:
                    if re.search(r'\bfor\s+.*\{', line) or re.search(r'\bwhile\s+.*\{', line):
                        nested_loops += 1
                        if nested_loops > 2:
                            hotspots.append({
                                'file': str(rust_file),
                                'function': 'unknown',
                                'issue': 'deeply_nested_loops',
                                'type': 'performance_risk'
                            })

                # Detect string operations in loops (allocation hotspot)
                for i, line in enumerate(lines):
                    if ('for ' in line or 'while ' in line) and i < len(lines) - 5:
                        loop_body = '\n'.join(lines[i:i+5])
                        if ('String::new' in loop_body or '.to_string()' in loop_body or
                            'format!' in loop_body):
                            hotspots.append({
                                'file': str(rust_file),
                                'line': i + 1,
                                'issue': 'string_allocation_in_loop',
                                'type': 'memory_hotspot'
                            })

        except Exception as e:
            print(f"Warning: Could not analyze {rust_file}: {e}")

    return hotspots

def analyze_dependency_bottlenecks():
    """Analyze dependency compilation times"""
    try:
        # Run cargo build with timing info
        result = subprocess.run([
            'cargo', 'build', '--timings=json', '--quiet'
        ], capture_output=True, text=True, timeout=60)

        if result.returncode == 0:
            return "Dependency analysis completed"
        else:
            return f"Dependency analysis failed: {result.stderr}"
    except subprocess.TimeoutExpired:
        return "Dependency analysis timed out"
    except Exception as e:
        return f"Dependency analysis error: {e}"

def generate_optimization_recommendations(hotspots):
    """Generate specific optimization recommendations"""
    recommendations = []

    for hotspot in hotspots:
        if hotspot['type'] == 'large_function':
            recommendations.append(f"🔥 {hotspot['file']}::{hotspot['function']} has {hotspot['lines']} lines - consider extracting helper functions")

        elif hotspot['type'] == 'performance_risk':
            recommendations.append(f"⚠️ {hotspot['file']} has deeply nested loops - consider algorithm optimization")

        elif hotspot['type'] == 'memory_hotspot':
            recommendations.append(f"💾 {hotspot['file']}:{hotspot['line']} has string allocation in loop - consider pre-allocation or string pooling")

    if not recommendations:
        recommendations.append("✅ No critical performance hotspots detected")

    return recommendations

def main():
    print("Hot Path Analysis for Ruchy Compiler")
    print("=" * 50)

    # Analyze source complexity
    print("🔍 Analyzing source complexity...")
    hotspots = analyze_source_complexity()

    print(f"Found {len(hotspots)} potential hotspots")

    # Analyze dependencies
    print("\n🔍 Analyzing dependency compilation...")
    dep_analysis = analyze_dependency_bottlenecks()
    print(dep_analysis)

    # Generate recommendations
    print("\n📋 Optimization Recommendations:")
    recommendations = generate_optimization_recommendations(hotspots)
    for rec in recommendations:
        print(f"  {rec}")

    # Write detailed report
    with open("target/optimization/hotspot-analysis.json", "w") as f:
        import json
        json.dump({
            "hotspots": hotspots,
            "recommendations": recommendations,
            "dependency_analysis": dep_analysis
        }, f, indent=2)

    print(f"\n📊 Detailed analysis saved to target/optimization/hotspot-analysis.json")

if __name__ == "__main__":
    main()
EOF

chmod +x target/optimization/detect-hotpaths.py
python3 target/optimization/detect-hotpaths.py

echo -e "${YELLOW}Phase 3: WASM binary size optimization${NC}"

# Create WASM optimization script
cat > target/optimization/optimize-wasm.sh << 'EOF'
#!/bin/bash
# WASM binary size optimization

echo "🎯 WASM Binary Size Optimization"

# Build WASM with different optimization levels
mkdir -p target/wasm-optimization

echo "Building with default settings..."
if command -v wasm-pack &> /dev/null; then
    wasm-pack build --target web --out-dir target/wasm-optimization/default >/dev/null 2>&1 || echo "Default build failed"

    echo "Building with size optimization..."
    wasm-pack build --target web --out-dir target/wasm-optimization/size -- --profile release-dist >/dev/null 2>&1 || echo "Size optimization build failed"

    # Check if wasm-opt is available for further optimization
    if command -v wasm-opt &> /dev/null; then
        echo "Applying wasm-opt optimizations..."
        find target/wasm-optimization -name "*.wasm" -exec wasm-opt -Os {} -o {}.opt \; 2>/dev/null || true
    else
        echo "wasm-opt not available for additional optimization"
    fi

    echo "Size comparison:"
    find target/wasm-optimization -name "*.wasm*" -exec ls -lh {} \; | awk '{print $5, $9}' | sort -k2
else
    echo "wasm-pack not available"
fi
EOF

chmod +x target/optimization/optimize-wasm.sh
./target/optimization/optimize-wasm.sh > target/optimization/wasm-optimization.log 2>&1

echo -e "${YELLOW}Phase 4: Memory allocation optimization${NC}"

# Create memory profiling script
cat > target/optimization/memory-profiling.ruchy << 'EOF'
// Memory allocation stress test for optimization analysis
let start_time = std::time::now()

// Test 1: String operations
let strings = []
for i in 0..100 {
    strings.push("Test string " + i.to_string())
}

// Test 2: Array operations
let numbers = []
for i in 0..1000 {
    numbers.push(i * 2)
}

// Test 3: Object creation
let objects = []
for i in 0..50 {
    objects.push({
        id: i,
        name: "Object " + i.to_string(),
        data: [i, i*2, i*3]
    })
}

let end_time = std::time::now()
println("Memory stress test completed in " + (end_time - start_time).to_string() + "ms")
EOF

echo -e "${YELLOW}Running memory allocation analysis...${NC}"
if cargo run --bin ruchy -- target/optimization/memory-profiling.ruchy > target/optimization/memory-profile.log 2>&1; then
    echo -e "${GREEN}✓ Memory profiling completed${NC}"
else
    echo -e "${YELLOW}Warning: Memory profiling failed${NC}"
fi

echo -e "${YELLOW}Phase 5: Compilation cache optimization${NC}"

# Analyze incremental compilation effectiveness
cat > target/optimization/analyze-incremental.sh << 'EOF'
#!/bin/bash
echo "🔄 Incremental Compilation Analysis"

# Clean build timing
echo "Testing clean build..."
cargo clean >/dev/null 2>&1
time cargo build --quiet >/dev/null 2>&1 || echo "Clean build failed"

# Incremental build timing
echo "Testing incremental build..."
touch src/lib.rs  # Trigger minimal rebuild
time cargo build --quiet >/dev/null 2>&1 || echo "Incremental build failed"

# Analyze target directory size
echo "Target directory analysis:"
du -sh target/ 2>/dev/null || echo "Cannot analyze target directory"

# Check for excessive intermediate files
echo "Intermediate file analysis:"
find target -name "*.rlib" -o -name "*.rmeta" | wc -l | xargs echo "Intermediate files:"
EOF

chmod +x target/optimization/analyze-incremental.sh
./target/optimization/analyze-incremental.sh > target/optimization/incremental-analysis.log 2>&1

echo -e "${YELLOW}Phase 6: Generate optimization report${NC}"

cat > target/optimization/optimization-report.md << EOF
# Critical Path Optimization Report

Generated: $(date)

## Executive Summary

This report analyzes critical performance paths in the Ruchy compiler and provides
optimization recommendations for improved compilation speed and runtime performance.

## Analysis Results

### 1. Hot Path Analysis
$(cat target/optimization/hotspot-analysis.json 2>/dev/null | python3 -m json.tool | grep -A 10 "recommendations" || echo "Hot path analysis data not available")

### 2. Compilation Performance
\`\`\`
$(cat target/optimization/compilation-times.log 2>/dev/null || echo "Compilation timing data not available")
\`\`\`

### 3. WASM Binary Optimization
\`\`\`
$(cat target/optimization/wasm-optimization.log 2>/dev/null || echo "WASM optimization data not available")
\`\`\`

### 4. Memory Profiling
\`\`\`
$(cat target/optimization/memory-profile.log 2>/dev/null || echo "Memory profiling data not available")
\`\`\`

### 5. Incremental Compilation
\`\`\`
$(cat target/optimization/incremental-analysis.log 2>/dev/null || echo "Incremental compilation data not available")
\`\`\`

## Optimization Recommendations

### High Priority
1. **Function Decomposition**: Large functions (>50 lines) should be split into smaller, focused functions
2. **Loop Optimization**: Reduce string allocations within loops using pre-allocation strategies
3. **Memory Management**: Implement object pooling for frequently allocated structures

### Medium Priority
1. **Dependency Optimization**: Review heavy dependencies and consider alternatives
2. **Incremental Compilation**: Ensure minimal rebuilds on small changes
3. **WASM Size**: Apply wasm-opt post-processing for production builds

### Low Priority
1. **Code Generation**: Profile generated Rust code for optimization opportunities
2. **Cache Strategies**: Implement intelligent caching for AST and type information
3. **Parallel Processing**: Consider parallelizing independent compilation phases

## Performance Targets

- **Compilation Speed**: <2s for incremental builds, <30s for clean builds
- **WASM Binary Size**: <500KB optimized
- **Memory Usage**: <100MB peak during compilation
- **Runtime Performance**: <10ms for typical REPL operations

## Implementation Priority

1. Address hot path functions identified in analysis
2. Implement memory optimization strategies
3. Optimize WASM build pipeline
4. Profile and optimize critical algorithms

## Monitoring

Use the following commands to monitor optimization progress:
- \`./scripts/performance-regression.sh\` - Detect performance regressions
- \`./scripts/critical-path-optimization.sh\` - Re-analyze hot paths
- \`cargo bench\` - Track benchmark improvements

EOF

# Check if any critical issues were found
CRITICAL_ISSUES=0

# Check for large functions
if grep -q "large_function" target/optimization/hotspot-analysis.json 2>/dev/null; then
    echo -e "${YELLOW}⚠️ Large functions detected - consider refactoring${NC}"
    CRITICAL_ISSUES=$((CRITICAL_ISSUES + 1))
fi

# Check compilation times
if grep -q "failed" target/optimization/compilation-times.log 2>/dev/null; then
    echo -e "${RED}❌ Compilation failures detected${NC}"
    CRITICAL_ISSUES=$((CRITICAL_ISSUES + 1))
fi

# Summary
if [ $CRITICAL_ISSUES -eq 0 ]; then
    echo -e "${GREEN}✅ Critical path optimization analysis completed - no critical issues${NC}"
else
    echo -e "${YELLOW}⚠️ Critical path optimization found $CRITICAL_ISSUES issues to address${NC}"
    echo -e "${YELLOW}Review target/optimization/optimization-report.md for details${NC}"
fi

echo -e "${GREEN}Critical path optimization analysis complete${NC}"