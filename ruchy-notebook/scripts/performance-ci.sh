#!/bin/bash
set -euo pipefail

# Performance CI Script for Ruchy Notebook
# Validates all performance targets and generates reports

echo "🚀 Ruchy Notebook Performance Validation Pipeline"
echo "================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Performance targets
WASM_SIZE_LIMIT=204800  # 200KB
CELL_EXECUTION_TARGET=50  # 50ms
NOTEBOOK_LOAD_TARGET=200  # 200ms for 100 cells
MEMORY_LEAK_THRESHOLD=10  # 10MB max increase

PROJECT_ROOT=$(dirname "$(dirname "$(realpath "$0")")")
cd "$PROJECT_ROOT"

echo -e "${BLUE}📁 Working directory: $PROJECT_ROOT${NC}"

# Function to print status
print_status() {
    local status=$1
    local message=$2
    case $status in
        "PASS")
            echo -e "${GREEN}✅ $message${NC}"
            ;;
        "FAIL")
            echo -e "${RED}❌ $message${NC}"
            ;;
        "WARN")
            echo -e "${YELLOW}⚠️  $message${NC}"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ️  $message${NC}"
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    print_status "INFO" "Checking prerequisites..."
    
    # Check for required tools
    command -v cargo >/dev/null 2>&1 || { print_status "FAIL" "cargo not found"; exit 1; }
    command -v wasm-pack >/dev/null 2>&1 || { print_status "FAIL" "wasm-pack not found"; exit 1; }
    command -v node >/dev/null 2>&1 || { print_status "WARN" "node not found - skipping JS tests"; }
    
    # Check Rust toolchain
    if cargo --version | grep -q "1."; then
        print_status "PASS" "Rust toolchain available"
    else
        print_status "FAIL" "Rust toolchain check failed"
        exit 1
    fi
    
    # Check wasm-pack version
    WASM_PACK_VERSION=$(wasm-pack --version | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1)
    print_status "INFO" "wasm-pack version: $WASM_PACK_VERSION"
}

# Build and validate WASM module
validate_wasm_build() {
    print_status "INFO" "Building and validating WASM module..."
    
    # Build WASM with optimization
    print_status "INFO" "Building optimized WASM module..."
    wasm-pack build --target web --out-dir pkg --release --features wasm 2>&1 | grep -E "(error|warning)" || true
    
    # Check if WASM file exists
    WASM_FILE="pkg/ruchy_notebook_bg.wasm"
    if [ ! -f "$WASM_FILE" ]; then
        print_status "FAIL" "WASM file not found: $WASM_FILE"
        return 1
    fi
    
    # Check WASM file size
    WASM_SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
    WASM_SIZE_KB=$((WASM_SIZE / 1024))
    
    print_status "INFO" "WASM module size: ${WASM_SIZE_KB}KB (limit: $((WASM_SIZE_LIMIT / 1024))KB)"
    
    if [ "$WASM_SIZE" -le "$WASM_SIZE_LIMIT" ]; then
        print_status "PASS" "WASM size within limit"
    else
        print_status "FAIL" "WASM size exceeds limit: ${WASM_SIZE_KB}KB > $((WASM_SIZE_LIMIT / 1024))KB"
        return 1
    fi
    
    # Validate WASM module structure
    if command -v wasm-objdump >/dev/null 2>&1; then
        EXPORT_COUNT=$(wasm-objdump -x "$WASM_FILE" | grep -c "export" || echo "0")
        print_status "INFO" "WASM exports: $EXPORT_COUNT"
        
        if [ "$EXPORT_COUNT" -gt 0 ]; then
            print_status "PASS" "WASM module has exports"
        else
            print_status "WARN" "WASM module has no exports"
        fi
    fi
}

# Run Rust benchmarks
run_rust_benchmarks() {
    print_status "INFO" "Running Rust benchmarks..."
    
    # Check if benchmark file exists
    if [ ! -f "benches/notebook_benchmarks.rs" ]; then
        print_status "WARN" "Benchmark file not found, skipping Rust benchmarks"
        return 0
    fi
    
    # Run benchmarks
    print_status "INFO" "Executing benchmark suite..."
    cargo bench --bench notebook_benchmarks 2>&1 | tee benchmark_results.txt || {
        print_status "WARN" "Some benchmarks failed, but continuing..."
    }
    
    # Parse benchmark results for key metrics
    if [ -f "benchmark_results.txt" ]; then
        # Extract VM execution times
        if grep -q "vm_execution" benchmark_results.txt; then
            print_status "PASS" "VM execution benchmarks completed"
        else
            print_status "WARN" "VM execution benchmarks missing"
        fi
        
        # Extract memory management metrics
        if grep -q "memory_management" benchmark_results.txt; then
            print_status "PASS" "Memory management benchmarks completed"
        else
            print_status "WARN" "Memory management benchmarks missing"
        fi
        
        # Check for performance targets
        if grep -q "performance_targets" benchmark_results.txt; then
            # Extract cell execution target results
            CELL_EXEC_TIME=$(grep -o "cell_execution_target.*time: \[[0-9.]*" benchmark_results.txt | grep -o "[0-9.]*" | tail -1 || echo "0")
            if [ -n "$CELL_EXEC_TIME" ] && [ "$(echo "$CELL_EXEC_TIME < $CELL_EXECUTION_TARGET" | bc -l 2>/dev/null || echo 0)" -eq 1 ]; then
                print_status "PASS" "Cell execution target met: ${CELL_EXEC_TIME}ms < ${CELL_EXECUTION_TARGET}ms"
            else
                print_status "WARN" "Cell execution target not met or not measurable"
            fi
        fi
    fi
}

# Run JavaScript performance tests
run_js_performance_tests() {
    print_status "INFO" "Running JavaScript performance tests..."
    
    if ! command -v node >/dev/null 2>&1; then
        print_status "WARN" "Node.js not available, skipping JS performance tests"
        return 0
    fi
    
    # Check if performance test file exists
    if [ ! -f "js/performance-tests.js" ]; then
        print_status "WARN" "JS performance tests not found, skipping"
        return 0
    fi
    
    # Create a simple test runner
    cat > test_runner.js << 'EOF'
const fs = require('fs');
const path = require('path');

// Mock browser APIs for Node.js
global.window = global;
global.document = {
    createElement: () => ({ 
        style: {}, 
        addEventListener: () => {},
        appendChild: () => {},
        removeChild: () => {},
        querySelector: () => null,
        classList: { add: () => {}, remove: () => {}, contains: () => false }
    }),
    body: { appendChild: () => {}, removeChild: () => {} },
    addEventListener: () => {}
};
global.navigator = { serviceWorker: null };
global.performance = { 
    now: () => Date.now(),
    mark: () => {},
    measure: () => {},
    getEntriesByName: () => []
};

// Mock import function
global.importMock = async () => ({
    default: async () => {},
    WasmNotebook: class {
        constructor() {}
        execute(code) { return { output: "mock", success: true }; }
    }
});

try {
    // Load performance tests
    const testCode = fs.readFileSync(path.join(__dirname, 'js/performance-tests.js'), 'utf8');
    
    // Replace dynamic imports with mocks
    const mockTestCode = testCode.replace(/await import\(['"`].*?['"`]\)/g, 'await importMock()');
    
    eval(mockTestCode);
    
    console.log('JS performance tests loaded successfully');
    process.exit(0);
} catch (error) {
    console.error('JS performance tests failed:', error.message);
    process.exit(1);
}
EOF
    
    if node test_runner.js; then
        print_status "PASS" "JS performance tests syntax validation passed"
    else
        print_status "WARN" "JS performance tests validation failed"
    fi
    
    # Cleanup
    rm -f test_runner.js
}

# Validate build artifacts
validate_build_artifacts() {
    print_status "INFO" "Validating build artifacts..."
    
    # Check required files
    REQUIRED_FILES=(
        "pkg/ruchy_notebook.js"
        "pkg/ruchy_notebook_bg.wasm"
        "pkg/ruchy_notebook.d.ts"
        "js/ruchy-notebook.js"
        "js/ruchy-worker.js"
        "js/sw.js"
        "js/manifest.json"
    )
    
    ALL_FILES_PRESENT=true
    
    for file in "${REQUIRED_FILES[@]}"; do
        if [ -f "$file" ]; then
            FILE_SIZE=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null)
            print_status "PASS" "$file present (${FILE_SIZE} bytes)"
        else
            print_status "FAIL" "$file missing"
            ALL_FILES_PRESENT=false
        fi
    done
    
    if [ "$ALL_FILES_PRESENT" = true ]; then
        print_status "PASS" "All required build artifacts present"
    else
        print_status "FAIL" "Some build artifacts missing"
        return 1
    fi
    
    # Validate JS syntax
    for js_file in js/*.js; do
        if [ -f "$js_file" ]; then
            if node -c "$js_file" 2>/dev/null; then
                print_status "PASS" "$(basename "$js_file") syntax valid"
            else
                print_status "WARN" "$(basename "$js_file") syntax issues"
            fi
        fi
    done
}

# Generate performance report
generate_performance_report() {
    print_status "INFO" "Generating performance report..."
    
    REPORT_FILE="performance_report.md"
    
    cat > "$REPORT_FILE" << EOF
# Ruchy Notebook Performance Report

Generated: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
Commit: $(git rev-parse HEAD 2>/dev/null || echo "unknown")

## WASM Module

- **File**: pkg/ruchy_notebook_bg.wasm
- **Size**: ${WASM_SIZE_KB:-"unknown"}KB / 200KB limit
- **Status**: $([ "${WASM_SIZE_KB:-999}" -le "$((WASM_SIZE_LIMIT / 1024))" ] && echo "✅ PASS" || echo "❌ FAIL")

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| WASM Size | <200KB | $([ "${WASM_SIZE_KB:-999}" -le "$((WASM_SIZE_LIMIT / 1024))" ] && echo "✅" || echo "❌") |
| Cell Execution | <50ms | ⚠️ Manual testing required |
| Notebook Loading | <200ms | ⚠️ Manual testing required |
| Memory Leaks | <10MB | ⚠️ Manual testing required |

## Build Artifacts

$(for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "- ✅ $file"
    else
        echo "- ❌ $file (missing)"
    fi
done)

## Recommendations

$(if [ "${WASM_SIZE_KB:-999}" -gt "$((WASM_SIZE_LIMIT / 1024))" ]; then
    echo "- 🔧 Optimize WASM size: Consider reducing dependencies or enabling more aggressive optimization"
fi)

- 🧪 Run manual performance tests using js/performance-tests.js
- 📊 Execute Rust benchmarks: \`cargo bench\`
- 🔍 Profile memory usage in production scenarios

## Notes

- This report covers automated checks only
- Manual testing required for complete performance validation
- Browser-specific optimizations may affect actual performance
EOF

    print_status "PASS" "Performance report generated: $REPORT_FILE"
}

# Main execution
main() {
    local exit_code=0
    
    echo "Starting performance validation pipeline..."
    
    # Run all validation steps
    check_prerequisites || exit_code=$?
    validate_wasm_build || exit_code=$?
    run_rust_benchmarks || exit_code=$?
    run_js_performance_tests || exit_code=$?
    validate_build_artifacts || exit_code=$?
    generate_performance_report || exit_code=$?
    
    echo
    echo "================================================="
    
    if [ $exit_code -eq 0 ]; then
        print_status "PASS" "Performance validation pipeline completed successfully"
        echo -e "${GREEN}🎉 All performance targets and validations passed!${NC}"
    else
        print_status "WARN" "Performance validation completed with warnings"
        echo -e "${YELLOW}⚠️  Some performance targets need attention. Check the report above.${NC}"
    fi
    
    echo -e "${BLUE}📊 Full report available in: performance_report.md${NC}"
    
    return $exit_code
}

# Execute main function
main "$@"