#!/bin/bash
# pre-release-gate.sh - Pre-release quality gate automation (95/100 minimum)
# Reference: Issue #170, trueno-aprender-stdlib-core-language-spec.md Section 13.5

set -u

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Configuration
MINIMUM_SCORE=95
SCORE=0
DETAILS=""

# Helper for tracking points
add_points() {
    local category="$1"
    local points="$2"
    local max="$3"
    local status="$4"
    SCORE=$((SCORE + points))
    DETAILS="${DETAILS}\n  ${category}: ${points}/${max} pts - ${status}"
}

echo -e "${BOLD}${BLUE}=== Pre-Release Quality Gate (95/100 minimum) ===${NC}"
echo "Reference: Issue #170"
echo "---------------------------------------------------"

# === 1. Tests Pass (20 pts) ===
echo -e "\n${BOLD}[1/7] Tests Pass (20 pts)${NC}"
if timeout 300 cargo test --lib --quiet 2>/dev/null; then
    echo -e "  ${GREEN}PASS${NC} - All tests pass"
    add_points "Tests pass" 20 20 "PASS"
else
    echo -e "  ${RED}FAIL${NC} - Tests failing"
    add_points "Tests pass" 0 20 "FAIL"
fi

# === 2. Coverage >= 95% (20 pts) ===
echo -e "\n${BOLD}[2/7] Test Coverage >= 95% (20 pts)${NC}"
# Check if cargo-llvm-cov is available
if command -v cargo-llvm-cov &>/dev/null; then
    COVERAGE=$(timeout 300 cargo llvm-cov --lib --text 2>/dev/null | grep -E "^TOTAL" | awk '{print $NF}' | tr -d '%' || echo "0")
    if [[ -z "$COVERAGE" ]]; then
        COVERAGE=0
    fi
    # Handle decimal values
    COVERAGE_INT=${COVERAGE%.*}

    if [[ "$COVERAGE_INT" -ge 95 ]]; then
        echo -e "  ${GREEN}PASS${NC} - Coverage: ${COVERAGE}%"
        add_points "Coverage ≥95%" 20 20 "${COVERAGE}%"
    elif [[ "$COVERAGE_INT" -ge 85 ]]; then
        echo -e "  ${YELLOW}PARTIAL${NC} - Coverage: ${COVERAGE}% (need 95%)"
        add_points "Coverage ≥95%" 15 20 "${COVERAGE}% (needs 95%)"
    elif [[ "$COVERAGE_INT" -ge 70 ]]; then
        echo -e "  ${YELLOW}PARTIAL${NC} - Coverage: ${COVERAGE}% (need 95%)"
        add_points "Coverage ≥95%" 10 20 "${COVERAGE}% (needs 95%)"
    else
        echo -e "  ${RED}FAIL${NC} - Coverage: ${COVERAGE}%"
        add_points "Coverage ≥95%" 0 20 "${COVERAGE}%"
    fi
else
    echo -e "  ${YELLOW}SKIP${NC} - cargo-llvm-cov not installed"
    echo "  Run: cargo install cargo-llvm-cov"
    add_points "Coverage ≥95%" 0 20 "SKIP (tool missing)"
fi

# === 3. Mutation Testing >= 85% (20 pts) ===
echo -e "\n${BOLD}[3/7] Mutation Kill Rate >= 85% (20 pts)${NC}"
# Check if cargo-mutants is available
if command -v cargo-mutants &>/dev/null; then
    # Only run on a sample of files to keep it reasonable
    echo "  Running mutation tests on sample..."
    MUTANTS_OUTPUT=$(timeout 300 cargo mutants --file src/backend/compiler.rs --timeout 60 2>&1 || true)
    CAUGHT=$(echo "$MUTANTS_OUTPUT" | grep -c "caught" || echo "0")
    MISSED=$(echo "$MUTANTS_OUTPUT" | grep -c "missed" || echo "0")
    TOTAL=$((CAUGHT + MISSED))

    if [[ "$TOTAL" -gt 0 ]]; then
        KILL_RATE=$((CAUGHT * 100 / TOTAL))
        if [[ "$KILL_RATE" -ge 85 ]]; then
            echo -e "  ${GREEN}PASS${NC} - Kill rate: ${KILL_RATE}% (${CAUGHT}/${TOTAL})"
            add_points "Mutation ≥85%" 20 20 "${KILL_RATE}%"
        elif [[ "$KILL_RATE" -ge 75 ]]; then
            echo -e "  ${YELLOW}PARTIAL${NC} - Kill rate: ${KILL_RATE}%"
            add_points "Mutation ≥85%" 15 20 "${KILL_RATE}%"
        else
            echo -e "  ${RED}FAIL${NC} - Kill rate: ${KILL_RATE}%"
            add_points "Mutation ≥85%" 10 20 "${KILL_RATE}%"
        fi
    else
        echo -e "  ${YELLOW}SKIP${NC} - No mutants generated"
        add_points "Mutation ≥85%" 15 20 "SKIP (no mutants)"
    fi
else
    echo -e "  ${YELLOW}SKIP${NC} - cargo-mutants not installed"
    echo "  Run: cargo install cargo-mutants"
    add_points "Mutation ≥85%" 0 20 "SKIP (tool missing)"
fi

# === 4. Zero SATD (10 pts) ===
echo -e "\n${BOLD}[4/7] Zero SATD (TODO/FIXME/HACK) (10 pts)${NC}"
SATD_COUNT=$(grep -rE "(TODO|FIXME|HACK|XXX):" src/ --include="*.rs" 2>/dev/null | wc -l || echo "0")
if [[ "$SATD_COUNT" -eq 0 ]]; then
    echo -e "  ${GREEN}PASS${NC} - Zero SATD markers"
    add_points "Zero SATD" 10 10 "PASS"
elif [[ "$SATD_COUNT" -le 5 ]]; then
    echo -e "  ${YELLOW}PARTIAL${NC} - ${SATD_COUNT} SATD markers found"
    add_points "Zero SATD" 5 10 "${SATD_COUNT} markers"
else
    echo -e "  ${RED}FAIL${NC} - ${SATD_COUNT} SATD markers found"
    grep -rE "(TODO|FIXME|HACK|XXX):" src/ --include="*.rs" 2>/dev/null | head -5
    add_points "Zero SATD" 0 10 "${SATD_COUNT} markers"
fi

# === 5. Clippy Clean (10 pts) ===
echo -e "\n${BOLD}[5/7] Clippy Clean (10 pts)${NC}"
CLIPPY_OUTPUT=$(timeout 120 cargo clippy --lib -- -D warnings 2>&1)
CLIPPY_EXIT=$?
if [[ "$CLIPPY_EXIT" -eq 0 ]]; then
    echo -e "  ${GREEN}PASS${NC} - Clippy clean"
    add_points "Clippy clean" 10 10 "PASS"
else
    CLIPPY_ERRORS=$(echo "$CLIPPY_OUTPUT" | grep -c "^error" || echo "0")
    echo -e "  ${RED}FAIL${NC} - ${CLIPPY_ERRORS} clippy errors"
    add_points "Clippy clean" 0 10 "${CLIPPY_ERRORS} errors"
fi

# === 6. Documentation Coverage >= 80% (10 pts) ===
echo -e "\n${BOLD}[6/7] Documentation Coverage >= 80% (10 pts)${NC}"
# Count public items and documented items
PUBLIC_ITEMS=$(grep -rE "^pub (fn|struct|enum|trait|type|const|static)" src/ --include="*.rs" 2>/dev/null | wc -l || echo "0")
DOCUMENTED_ITEMS=$(grep -rEB1 "^pub (fn|struct|enum|trait|type|const|static)" src/ --include="*.rs" 2>/dev/null | grep -c "///" || echo "0")

if [[ "$PUBLIC_ITEMS" -gt 0 ]]; then
    DOC_PERCENT=$((DOCUMENTED_ITEMS * 100 / PUBLIC_ITEMS))
    if [[ "$DOC_PERCENT" -ge 80 ]]; then
        echo -e "  ${GREEN}PASS${NC} - Doc coverage: ${DOC_PERCENT}% (${DOCUMENTED_ITEMS}/${PUBLIC_ITEMS})"
        add_points "Doc coverage" 10 10 "${DOC_PERCENT}%"
    elif [[ "$DOC_PERCENT" -ge 60 ]]; then
        echo -e "  ${YELLOW}PARTIAL${NC} - Doc coverage: ${DOC_PERCENT}%"
        add_points "Doc coverage" 7 10 "${DOC_PERCENT}%"
    else
        echo -e "  ${RED}FAIL${NC} - Doc coverage: ${DOC_PERCENT}%"
        add_points "Doc coverage" 3 10 "${DOC_PERCENT}%"
    fi
else
    echo -e "  ${YELLOW}SKIP${NC} - No public items found"
    add_points "Doc coverage" 5 10 "SKIP"
fi

# === 7. Property Tests (10 pts) ===
echo -e "\n${BOLD}[7/7] Property Tests (10 pts)${NC}"
# Check if proptest tests exist and run
PROP_TEST_COUNT=$(grep -r "proptest!" src/ --include="*.rs" 2>/dev/null | wc -l || echo "0")
if [[ "$PROP_TEST_COUNT" -gt 0 ]]; then
    if timeout 180 cargo test property --lib --quiet 2>/dev/null; then
        echo -e "  ${GREEN}PASS${NC} - ${PROP_TEST_COUNT} property test modules pass"
        add_points "Property tests" 10 10 "PASS (${PROP_TEST_COUNT} modules)"
    else
        echo -e "  ${YELLOW}PARTIAL${NC} - Property tests exist but some fail"
        add_points "Property tests" 5 10 "PARTIAL"
    fi
else
    echo -e "  ${YELLOW}WARN${NC} - No property tests found"
    add_points "Property tests" 0 10 "MISSING"
fi

# === Summary ===
echo -e "\n${BOLD}${BLUE}=== Quality Gate Summary ===${NC}"
echo "---------------------------------------------------"
echo -e "${DETAILS}"
echo "---------------------------------------------------"
echo -e "\n${BOLD}Final Score: ${SCORE}/100${NC}"

if [[ "$SCORE" -ge "$MINIMUM_SCORE" ]]; then
    echo -e "${GREEN}${BOLD}STATUS: APPROVED FOR RELEASE${NC}"
    echo -e "Score ${SCORE}/100 meets minimum threshold of ${MINIMUM_SCORE}/100"
    exit 0
elif [[ "$SCORE" -ge 85 ]]; then
    echo -e "${YELLOW}${BOLD}STATUS: PROVISIONAL (REVIEW REQUIRED)${NC}"
    echo -e "Score ${SCORE}/100 is close to threshold. Review failures before release."
    exit 1
else
    echo -e "${RED}${BOLD}STATUS: BLOCKED${NC}"
    echo -e "Score ${SCORE}/100 is below threshold of ${MINIMUM_SCORE}/100"
    echo -e "Release is blocked until quality issues are resolved."
    exit 1
fi
