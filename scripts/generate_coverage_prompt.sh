#!/usr/bin/env bash
# scripts/generate_coverage_prompt.sh
# Generates AI-ready coverage improvement prompts based on current state
# Quality: bashrs validated, DET002 exempt (intentional timestamps for logging)

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m' # No Color

# Constants
readonly SPEC_FILE="docs/specifications/90-percent-coverage-strategy-spec.md"
readonly LCOV_FILE="target/coverage/lcov.info"
readonly TARGET_COVERAGE=90.0
readonly CURRENT_DATE=$(date +%Y-%m-%d)

# Parse current coverage from LCOV
get_current_coverage() {
    if [[ ! -f "$LCOV_FILE" ]]; then
        echo "0.00"
        return
    fi

    awk -F: '
        BEGIN {lf=0; lh=0}
        /^LF:/ {lf += $2}
        /^LH:/ {lh += $2}
        END {
            if (lf > 0) {
                printf "%.2f", (lh / lf) * 100
            } else {
                print "0.00"
            }
        }
    ' "$LCOV_FILE"
}

# Calculate coverage gap
calculate_gap() {
    local current=$1
    awk -v target="$TARGET_COVERAGE" -v curr="$current" '
        BEGIN {printf "%.2f", target - curr}
    '
}

# Identify priority modules from LCOV
identify_priority_modules() {
    if [[ ! -f "$LCOV_FILE" ]]; then
        echo "  (Run 'make coverage' first to generate data)"
        return
    fi

    # Parse LCOV to find lowest coverage modules
    awk -F: '
        /^SF:/ {file = $2; gsub(/^.*\/src\//, "src/", file)}
        /^LH:/ {lh = $2}
        /^LF:/ {
            lf = $2
            if (lf > 0) {
                cov = (lh / lf) * 100
                if (cov < 80 && file ~ /src\//) {
                    printf "  - %s: %.1f%% coverage\n", file, cov
                }
            }
        }
    ' "$LCOV_FILE" | head -10
}

# Generate prompt based on phase
generate_phase1_prompt() {
    local current_coverage=$1
    local gap=$2

    cat <<EOF

${CYAN}═══════════════════════════════════════════════════════════════${NC}
${GREEN}PHASE 1: Low-Hanging Fruit (70% → 80%)${NC}
${CYAN}═══════════════════════════════════════════════════════════════${NC}

${YELLOW}📊 CURRENT STATE${NC}
  Coverage: ${current_coverage}%
  Target: 80.0%
  Gap: ${gap}%

${YELLOW}📋 TASK SELECTION${NC}

Choose one of the following tasks:

${BLUE}[Task 1.1]${NC} Inline Unit Tests
  - Add 10+ tests per module
  - Pattern: Section 2.2, Pattern 1 (bashrs: 13.5 tests/file)
  - Time: 5 min/test × 10 tests = 50 min per module
  - Priority modules:
$(identify_priority_modules)

${BLUE}[Task 1.2]${NC} Property Test Case Increase
  - Change PROPTEST_CASES=5 → PROPTEST_CASES=100
  - Location: Makefile line ~337
  - Time: 5 minutes
  - Impact: +3-5% coverage, statistical significance

${BLUE}[Task 1.3]${NC} Negative Test Cases
  - Test all Result<T, E> error paths
  - Pattern: Section 4.1, Task 1.3
  - Time: 3 min/test × ~500 Results = 25 hours (batch work)

${YELLOW}📝 READY-TO-USE PROMPT${NC}

Copy this prompt to start work:

${CYAN}─────────────────────────────────────────────────────────────${NC}
Task: Improve Ruchy compiler test coverage (Phase 1)

Context:
- Read docs/specifications/90-percent-coverage-strategy-spec.md
- Current coverage: ${current_coverage}% (target: >90%)
- Focus: Phase 1, Task 1.1 (inline unit tests)

Pattern to follow:
- bashrs pattern: 13.5 tests per file (Spec Section 2.2, Pattern 1)
- Test structure: Section 4.2.1 (error handling examples)
- Include property tests: PROPTEST_CASES=100 (Spec Section 2.2, Pattern 3)

Priority module: [SELECT FROM LIST ABOVE]

Success criteria:
- ≥10 inline unit tests added
- All Result<T,E> error paths tested
- Coverage increases by ≥5%
- Tests run in <10 min total

Start by analyzing the module's public API and untested code paths.
${CYAN}─────────────────────────────────────────────────────────────${NC}

EOF
}

generate_phase2_prompt() {
    local current_coverage=$1

    cat <<EOF

${CYAN}═══════════════════════════════════════════════════════════════${NC}
${GREEN}PHASE 2: Systematic Coverage (80% → 90%)${NC}
${CYAN}═══════════════════════════════════════════════════════════════${NC}

${YELLOW}📊 CURRENT STATE${NC}
  Coverage: ${current_coverage}%
  Target: 90.0%

${YELLOW}📋 TASK SELECTION${NC}

${BLUE}[Task 2.1]${NC} Untested Builtins Test Suite
  - 40 builtins from /tmp/untested_builtins.txt
  - Time: 40 × 5 tests × 10 min = 33 hours
  - Pattern: Section 4.2, Task 2.1

${BLUE}[Task 2.2]${NC} Transpiler Golden File Tests
  - Create tests/golden/ directory
  - 100 Ruchy → Rust pairs
  - Time: 100 files × 15 min = 25 hours
  - Pattern: Section 4.2, Task 2.2

${BLUE}[Task 2.3]${NC} WASM Mock Testing
  - Mock WASM APIs for unit tests
  - 5 modules × 20 tests = 100 tests
  - Time: 13 hours
  - Pattern: Section 4.2, Task 2.3

${YELLOW}📝 READY-TO-USE PROMPT${NC}

${CYAN}─────────────────────────────────────────────────────────────${NC}
Task: Systematic coverage improvement (Phase 2)

Context:
- Read docs/specifications/90-percent-coverage-strategy-spec.md
- Current coverage: ${current_coverage}%
- Focus: [SELECT TASK FROM ABOVE]

Implementation guide: Section 4.2 of spec

Success criteria:
- Task-specific metrics (see spec Section 4.2)
- Coverage increases by ≥5%
- All tests pass in <10 min
${CYAN}─────────────────────────────────────────────────────────────${NC}

EOF
}

generate_phase3_prompt() {
    local current_coverage=$1

    cat <<EOF

${CYAN}═══════════════════════════════════════════════════════════════${NC}
${GREEN}PHASE 3: Comprehensive Coverage (90% → 95%+)${NC}
${CYAN}═══════════════════════════════════════════════════════════════${NC}

${YELLOW}📊 CURRENT STATE${NC}
  Coverage: ${current_coverage}%
  Target: 95.0%

${YELLOW}📋 LONG-TERM TASKS${NC}

${BLUE}[Task 3.1]${NC} Mutation Testing at Scale
  - 542 source files × 5 min = 45 hours compute
  - Target: 75%+ mutation score
  - Pattern: Section 4.3, Task 3.1

${BLUE}[Task 3.2]${NC} Fuzz Testing for Parser
  - cargo-fuzz setup
  - 1 week continuous fuzzing
  - Pattern: Section 4.3, Task 3.2

${BLUE}[Task 3.3]${NC} Doctests for All Public APIs
  - 500 public functions × 5 min = 42 hours
  - Pattern: Section 4.3, Task 3.3

${YELLOW}📝 READY-TO-USE PROMPT${NC}

${CYAN}─────────────────────────────────────────────────────────────${NC}
Task: Comprehensive coverage improvement (Phase 3)

Context:
- Read docs/specifications/90-percent-coverage-strategy-spec.md
- Current coverage: ${current_coverage}%
- Focus: [SELECT TASK FROM ABOVE]

Implementation guide: Section 4.3 of spec

This is long-term work. Proceed systematically.
${CYAN}─────────────────────────────────────────────────────────────${NC}

EOF
}

# Main function
main() {
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}   Ruchy Coverage Improvement Prompt Generator${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""

    # Check if spec exists
    if [[ ! -f "$SPEC_FILE" ]]; then
        echo -e "${RED}ERROR: Specification file not found: $SPEC_FILE${NC}"
        echo "Run this script from the repository root."
        exit 1
    fi

    # Get current coverage
    local current_coverage
    current_coverage=$(get_current_coverage)

    if [[ "$current_coverage" == "0.00" ]]; then
        echo -e "${YELLOW}⚠️  No coverage data found. Run 'make coverage' first.${NC}"
        echo ""
    fi

    local gap
    gap=$(calculate_gap "$current_coverage")

    # Determine phase based on coverage
    if (( $(echo "$current_coverage < 80" | bc -l) )); then
        generate_phase1_prompt "$current_coverage" "$gap"
    elif (( $(echo "$current_coverage < 90" | bc -l) )); then
        generate_phase2_prompt "$current_coverage"
    else
        generate_phase3_prompt "$current_coverage"
    fi

    echo ""
    echo -e "${GREEN}✅ Prompt generated successfully${NC}"
    echo -e "${BLUE}📖 Full strategy: $SPEC_FILE${NC}"
    echo -e "${BLUE}📊 Coverage data: $LCOV_FILE${NC}"
    echo ""
    echo -e "${YELLOW}💡 TIP: Copy the prompt above and paste into Claude Code${NC}"
    echo ""
}

# Run main
main "$@"
