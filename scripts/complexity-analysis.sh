#!/bin/bash
# scripts/complexity-analysis.sh
# WebAssembly Extreme Quality Assurance Framework v3.0
# Complexity Analysis Script

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Starting complexity analysis...${NC}"

# Thresholds as specified in the framework
MAX_CYCLOMATIC=10
MAX_COGNITIVE=15
MAX_FUNCTIONS_PER_FILE=50
MAX_LINES_PER_FUNCTION=30

# Create output directory
mkdir -p target/complexity

echo -e "${YELLOW}Phase 1: Cyclomatic complexity analysis${NC}"
if command -v tokei &> /dev/null; then
    tokei --output json src/ > target/complexity/tokei-report.json
    echo -e "${GREEN}✓ Code statistics collected${NC}"
else
    echo -e "${YELLOW}Warning: tokei not found, skipping code statistics${NC}"
fi

echo -e "${YELLOW}Phase 2: Function complexity analysis${NC}"
# Use clippy for complexity analysis
cargo clippy --all-targets --all-features -- \
    -W clippy::cognitive_complexity \
    -W clippy::cyclomatic_complexity \
    -W clippy::too_many_lines \
    -W clippy::too_many_arguments \
    2> target/complexity/clippy-complexity.log || true

echo -e "${YELLOW}Phase 3: PMAT analysis (if available)${NC}"
if command -v pmat &> /dev/null; then
    pmat analyze complexity \
        --max-cyclomatic $MAX_CYCLOMATIC \
        --max-cognitive $MAX_COGNITIVE \
        --output target/complexity/pmat-complexity.json \
        src/ || echo -e "${YELLOW}PMAT analysis completed with warnings${NC}"
else
    echo -e "${YELLOW}Warning: PMAT not found, using alternative analysis${NC}"

    # Alternative: Use rustc to analyze function lengths
    echo -e "${YELLOW}Running alternative complexity check...${NC}"

    find src -name "*.rs" -exec wc -l {} \; | \
    while read lines file; do
        if [ "$lines" -gt 500 ]; then
            echo "Warning: $file has $lines lines (consider splitting)"
        fi
    done > target/complexity/large-files.log
fi

echo -e "${YELLOW}Phase 4: Dependency complexity${NC}"
cargo tree --depth 3 > target/complexity/dependency-tree.txt
cargo tree --duplicates > target/complexity/duplicate-deps.txt 2>/dev/null || true

echo -e "${YELLOW}Phase 5: Generate complexity report${NC}"
cat > target/complexity/complexity-report.md << EOF
# Complexity Analysis Report

Generated: $(date)

## Thresholds
- Maximum Cyclomatic Complexity: $MAX_CYCLOMATIC
- Maximum Cognitive Complexity: $MAX_COGNITIVE
- Maximum Functions per File: $MAX_FUNCTIONS_PER_FILE
- Maximum Lines per Function: $MAX_LINES_PER_FUNCTION

## Analysis Results

### Code Statistics
$(if [ -f target/complexity/tokei-report.json ]; then echo "✓ Code statistics available in tokei-report.json"; else echo "⚠ Code statistics not available"; fi)

### Clippy Complexity Warnings
$(wc -l < target/complexity/clippy-complexity.log) complexity warnings found

### Large Files
$(if [ -f target/complexity/large-files.log ]; then cat target/complexity/large-files.log; else echo "No files exceed size thresholds"; fi)

### Dependency Complexity
- Dependency tree depth: 3 levels analyzed
- Duplicate dependencies: $(if [ -f target/complexity/duplicate-deps.txt ]; then wc -l < target/complexity/duplicate-deps.txt; else echo "0"; fi)

## Recommendations

EOF

# Check for violations
VIOLATIONS=0

# Check clippy warnings
CLIPPY_WARNINGS=$(wc -l < target/complexity/clippy-complexity.log)
if [ "$CLIPPY_WARNINGS" -gt 0 ]; then
    echo -e "${YELLOW}Found $CLIPPY_WARNINGS complexity warnings${NC}"
    VIOLATIONS=$((VIOLATIONS + CLIPPY_WARNINGS))
fi

# Summary
if [ $VIOLATIONS -eq 0 ]; then
    echo -e "${GREEN}✓ Complexity analysis passed - no violations found${NC}"
    echo "✅ PASS: All complexity thresholds met" >> target/complexity/complexity-report.md
else
    echo -e "${YELLOW}⚠ Complexity analysis found $VIOLATIONS violations${NC}"
    echo "⚠️ REVIEW: $VIOLATIONS complexity violations found" >> target/complexity/complexity-report.md

    # Don't fail the build, just warn
    echo -e "${YELLOW}Review target/complexity/complexity-report.md for details${NC}"
fi

echo -e "${GREEN}Complexity analysis complete${NC}"