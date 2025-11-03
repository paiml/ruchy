#!/bin/bash
# OPT-GLOBAL-001: Profile-Guided Optimization Workflow Script
# GREEN Phase: Minimal implementation to support PGO testing
#
# Usage:
#   ./scripts/run-pgo.sh instrument  # Build with profile collection
#   ./scripts/run-pgo.sh collect     # Run workload to collect profiles
#   ./scripts/run-pgo.sh merge       # Merge profile data
#   ./scripts/run-pgo.sh optimize    # Build with PGO optimization
#   ./scripts/run-pgo.sh clean       # Clean up profile data

set -euo pipefail

# Configuration
PGO_DATA_DIR="/tmp/pgo-data-ruchy"
PROFILE_FILE="${PGO_DATA_DIR}/merged.profdata"
WORKLOAD_DIR="./examples"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[PGO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[PGO]${NC} $1"
}

log_error() {
    echo -e "${RED}[PGO]${NC} $1"
}

# Check for required tools
check_requirements() {
    if ! command -v llvm-profdata &> /dev/null; then
        log_error "llvm-profdata not found. Install LLVM tools:"
        log_error "  Ubuntu/Debian: apt-get install llvm"
        log_error "  macOS: brew install llvm"
        exit 1
    fi
    log_info "Requirements check passed"
}

# Step 1: Build with profile instrumentation
instrument() {
    log_info "Building with profile instrumentation..."
    mkdir -p "${PGO_DATA_DIR}"

    export RUSTFLAGS="-Cprofile-generate=${PGO_DATA_DIR}"
    cargo build --release --bin ruchy

    log_info "Instrumented build complete: target/release/ruchy"
    log_info "Run './scripts/run-pgo.sh collect' to collect profile data"
}

# Step 2: Collect profile data by running workload
collect() {
    log_info "Collecting profile data..."

    if [ ! -f "target/release/ruchy" ]; then
        log_error "Instrumented binary not found. Run './scripts/run-pgo.sh instrument' first"
        exit 1
    fi

    # Set environment variable for profile output
    export LLVM_PROFILE_FILE="${PGO_DATA_DIR}/ruchy-%p.profraw"

    # Run representative workload - transpile all examples
    local count=0
    for example in "${WORKLOAD_DIR}"/*.ruchy; do
        if [ -f "$example" ]; then
            log_info "Processing: $(basename "$example")"
            timeout 10 ./target/release/ruchy transpile "$example" > /dev/null || true
            ((count++))
        fi
    done

    log_info "Collected profile data from ${count} examples"
    log_info "Profile files: ${PGO_DATA_DIR}/*.profraw"

    # Verify profile data was created
    local profraw_count=$(ls -1 "${PGO_DATA_DIR}"/*.profraw 2>/dev/null | wc -l)
    if [ "$profraw_count" -eq 0 ]; then
        log_error "No profile data collected. Check that instrumented binary ran successfully"
        exit 1
    fi

    log_info "Found ${profraw_count} profile data files"
    log_info "Run './scripts/run-pgo.sh merge' to merge profile data"
}

# Step 3: Merge profile data
merge() {
    log_info "Merging profile data..."

    check_requirements

    local profraw_count=$(ls -1 "${PGO_DATA_DIR}"/*.profraw 2>/dev/null | wc -l)
    if [ "$profraw_count" -eq 0 ]; then
        log_error "No .profraw files found in ${PGO_DATA_DIR}"
        log_error "Run './scripts/run-pgo.sh collect' first"
        exit 1
    fi

    log_info "Merging ${profraw_count} profile files..."
    llvm-profdata merge \
        -output="${PROFILE_FILE}" \
        "${PGO_DATA_DIR}"/*.profraw

    if [ -f "${PROFILE_FILE}" ]; then
        local size=$(du -h "${PROFILE_FILE}" | cut -f1)
        log_info "Merged profile: ${PROFILE_FILE} (${size})"
        log_info "Run './scripts/run-pgo.sh optimize' to build optimized binary"
    else
        log_error "Profile merge failed"
        exit 1
    fi
}

# Step 4: Build with PGO optimization
optimize() {
    log_info "Building with PGO optimization..."

    if [ ! -f "${PROFILE_FILE}" ]; then
        log_error "Merged profile not found: ${PROFILE_FILE}"
        log_error "Run './scripts/run-pgo.sh merge' first"
        exit 1
    fi

    export RUSTFLAGS="-Cprofile-use=${PROFILE_FILE} -Cllvm-args=-pgo-warn-missing-function"
    cargo build --release --bin ruchy

    log_info "PGO-optimized build complete: target/release/ruchy"
    log_info "Expected speedup: 15-30% (per OPT-GLOBAL-001 spec)"
}

# Clean up profile data
clean() {
    log_info "Cleaning up profile data..."

    if [ -d "${PGO_DATA_DIR}" ]; then
        rm -rf "${PGO_DATA_DIR}"
        log_info "Removed: ${PGO_DATA_DIR}"
    fi

    log_info "Cleanup complete"
}

# Full workflow
full() {
    log_info "Running full PGO workflow..."
    instrument
    collect
    merge
    optimize
    log_info "PGO workflow complete!"
    log_info "PGO-optimized binary: target/release/ruchy"
}

# Main dispatch
case "${1:-help}" in
    instrument)
        instrument
        ;;
    collect)
        collect
        ;;
    merge)
        merge
        ;;
    optimize)
        optimize
        ;;
    clean)
        clean
        ;;
    full)
        full
        ;;
    help|--help|-h)
        echo "Usage: $0 {instrument|collect|merge|optimize|clean|full}"
        echo ""
        echo "Commands:"
        echo "  instrument  - Build with profile collection instrumentation"
        echo "  collect     - Run workload to collect profile data"
        echo "  merge       - Merge profile data files"
        echo "  optimize    - Build with PGO optimization"
        echo "  clean       - Remove profile data"
        echo "  full        - Run complete workflow (instrument→collect→merge→optimize)"
        echo ""
        echo "Example workflow:"
        echo "  ./scripts/run-pgo.sh instrument"
        echo "  ./scripts/run-pgo.sh collect"
        echo "  ./scripts/run-pgo.sh merge"
        echo "  ./scripts/run-pgo.sh optimize"
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Run '$0 help' for usage"
        exit 1
        ;;
esac
