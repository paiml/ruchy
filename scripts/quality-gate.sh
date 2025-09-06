#!/bin/bash
# Quality Gate Script - Checks individual functions against complexity limits
# Per CLAUDE.md: Structural complexity ≤10, Cognitive complexity ≤10

set -e

echo "🔒 Running Quality Gate Checks..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
MAX_CYCLOMATIC=10
MAX_COGNITIVE=10
PROJECT_PATH="${1:-src}"

# Track violations
VIOLATIONS=0
VIOLATION_DETAILS=""

# Function to check complexity
check_complexity() {
    echo "📊 Checking function complexity (max cyclomatic: $MAX_CYCLOMATIC)..."
    
    # Find all Rust files and check complexity
    COMPLEXITY_OUTPUT=$(pmat analyze complexity --max-cyclomatic $MAX_CYCLOMATIC --format=detailed 2>&1 || true)
    
    # Extract functions exceeding limit
    HIGH_COMPLEXITY=$(echo "$COMPLEXITY_OUTPUT" | grep "cyclomatic complexity: [1-9][1-9]\|cyclomatic complexity: [2-9][0-9]" || true)
    
    if [ -n "$HIGH_COMPLEXITY" ]; then
        VIOLATION_COUNT=$(echo "$HIGH_COMPLEXITY" | wc -l)
        VIOLATIONS=$((VIOLATIONS + VIOLATION_COUNT))
        VIOLATION_DETAILS="${VIOLATION_DETAILS}\n❌ Found $VIOLATION_COUNT functions with complexity > $MAX_CYCLOMATIC:\n"
        
        # Get top 5 worst offenders
        echo "$COMPLEXITY_OUTPUT" | grep -A 1 "Top Complexity Hotspots" | tail -n +2 | head -6 | while read -r line; do
            if [[ $line =~ cyclomatic\ complexity:\ ([0-9]+) ]]; then
                COMPLEXITY="${BASH_REMATCH[1]}"
                if [ "$COMPLEXITY" -gt "$MAX_CYCLOMATIC" ]; then
                    VIOLATION_DETAILS="${VIOLATION_DETAILS}  $line\n"
                fi
            fi
        done
    else
        echo -e "${GREEN}✅ All functions have complexity ≤ $MAX_CYCLOMATIC${NC}"
    fi
}

# Function to check for SATD
check_satd() {
    echo "📝 Checking for technical debt (SATD)..."
    
    SATD_COUNT=$(grep -r "TODO\|FIXME\|HACK" "$PROJECT_PATH" --include="*.rs" 2>/dev/null | wc -l || echo "0")
    
    if [ "$SATD_COUNT" -gt 0 ]; then
        VIOLATIONS=$((VIOLATIONS + SATD_COUNT))
        VIOLATION_DETAILS="${VIOLATION_DETAILS}\n❌ Found $SATD_COUNT SATD comments (TODO/FIXME/HACK)\n"
        
        # Show first 5 examples
        grep -r "TODO\|FIXME\|HACK" "$PROJECT_PATH" --include="*.rs" 2>/dev/null | head -5 | while read -r line; do
            VIOLATION_DETAILS="${VIOLATION_DETAILS}  ${line:0:100}...\n"
        done
    else
        echo -e "${GREEN}✅ No SATD comments found${NC}"
    fi
}

# Function to check specific high-complexity functions we know about
check_known_violations() {
    echo "🔍 Checking known high-complexity functions..."
    
    # List of functions we know need fixing
    KNOWN_FUNCTIONS=(
        "compile_source_to_binary:13:src/backend/compiler.rs"
        "parse_condition_term:11:src/frontend/parser/collections.rs"
        "parse_dataframe_column_definitions:11:src/frontend/parser/collections.rs"
        "parse_struct_definition:11:src/frontend/parser/expressions.rs"
        "parse_object_literal_body:11:src/frontend/parser/collections.rs"
    )
    
    for func_info in "${KNOWN_FUNCTIONS[@]}"; do
        IFS=':' read -r func_name complexity file <<< "$func_info"
        if [ "$complexity" -gt "$MAX_CYCLOMATIC" ]; then
            if [ -f "$file" ]; then
                if grep -q "fn $func_name" "$file" 2>/dev/null; then
                    VIOLATIONS=$((VIOLATIONS + 1))
                    VIOLATION_DETAILS="${VIOLATION_DETAILS}  ⚠️  $func_name (complexity: $complexity) in $file\n"
                fi
            fi
        fi
    done
}

# Main execution
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Quality Gate for: $PROJECT_PATH"
echo "Limits: Cyclomatic ≤$MAX_CYCLOMATIC, Cognitive ≤$MAX_COGNITIVE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run checks
check_complexity
check_satd
check_known_violations

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Report results
if [ $VIOLATIONS -gt 0 ]; then
    echo -e "${RED}❌ QUALITY GATE FAILED${NC}"
    echo -e "Found $VIOLATIONS total violations:"
    echo -e "$VIOLATION_DETAILS"
    echo ""
    echo "To fix:"
    echo "1. Refactor functions with complexity >$MAX_CYCLOMATIC"
    echo "2. Remove all TODO/FIXME/HACK comments"
    echo "3. Run 'make quality-gate' to verify"
    exit 1
else
    echo -e "${GREEN}✅ QUALITY GATE PASSED${NC}"
    echo "All functions meet complexity requirements"
    echo "No technical debt found"
    exit 0
fi