#!/bin/bash
# TDG Transactional Tracking System
# Ensures technical debt never increases without explicit acknowledgment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Files to track
TDG_BASELINE=".tdg_baseline.json"
TDG_CURRENT=".tdg_current.json"
TDG_HISTORY=".tdg_history.log"

# Generate current TDG scores
echo "📊 Calculating current TDG scores..."
if ! timeout 90s pmat tdg . --format=json > "$TDG_CURRENT" 2>/dev/null; then
    echo "⚠️ TDG analysis failed, falling back to quality-gate"
    if ! timeout 90s pmat quality-gate --format=json > "$TDG_CURRENT" 2>/dev/null; then
        echo "❌ Both TDG and quality-gate failed, cannot proceed"
        exit 1
    fi
fi

# Check if baseline exists
if [ ! -f "$TDG_BASELINE" ]; then
    echo "⚠️ No TDG baseline found. Creating initial baseline..."
    cp "$TDG_CURRENT" "$TDG_BASELINE"
    echo "✅ Initial TDG baseline created"
    exit 0
fi

# Compare scores for modified files
MODIFIED_FILES=$(git diff --name-only --cached | grep '\.rs$' || true)

if [ -z "$MODIFIED_FILES" ]; then
    echo "ℹ️ No Rust files modified"
    exit 0
fi

echo "🔍 Checking TDG scores for modified files..."
VIOLATIONS=0
IMPROVEMENTS=0

for file in $MODIFIED_FILES; do
    # Get baseline score (handle both TDG and quality-gate JSON formats)
    BASELINE_SCORE=$(jq -r "if .files then (.files[] | select(.file_path == \"./$file\" or .path == \"$file\") | .total // .score // 100) else .overall_score // 100 end" "$TDG_BASELINE" 2>/dev/null || echo "100")
    BASELINE_GRADE=$(jq -r "if .files then (.files[] | select(.file_path == \"./$file\" or .path == \"$file\") | .grade // \"New\") else \"New\" end" "$TDG_BASELINE" 2>/dev/null || echo "New")
    
    # Get current score (handle both TDG and quality-gate JSON formats)
    CURRENT_SCORE=$(jq -r "if .files then (.files[] | select(.file_path == \"./$file\" or .path == \"$file\") | .total // .score // 100) else .overall_score // 100 end" "$TDG_CURRENT" 2>/dev/null || echo "100")
    CURRENT_GRADE=$(jq -r "if .files then (.files[] | select(.file_path == \"./$file\" or .path == \"$file\") | .grade // \"New\") else \"New\" end" "$TDG_CURRENT" 2>/dev/null || echo "New")
    
    # Skip if no score found for file (likely means it wasn't analyzed)
    if [ "$BASELINE_SCORE" = "null" ] || [ "$CURRENT_SCORE" = "null" ]; then
        echo "➖ $file: No TDG data (may be new or skipped)"
        continue
    fi
    
    # Calculate delta
    if command -v bc >/dev/null 2>&1; then
        DELTA=$(echo "$CURRENT_SCORE - $BASELINE_SCORE" | bc -l 2>/dev/null || echo "0")
    else
        DELTA=$(awk "BEGIN {printf \"%.2f\", $CURRENT_SCORE - $BASELINE_SCORE}")
    fi
    
    # Format output
    if [ -n "$DELTA" ] && ([ "$DELTA" != "0" ] && [ "${DELTA%.*}" -lt "0" ]); then
        echo -e "${RED}❌ $file: TDG degraded${NC}"
        echo "   Baseline: $BASELINE_SCORE ($BASELINE_GRADE) → Current: $CURRENT_SCORE ($CURRENT_GRADE)"
        echo "   Delta: $DELTA"
        VIOLATIONS=$((VIOLATIONS + 1))
    elif [ -n "$DELTA" ] && ([ "$DELTA" != "0" ] && [ "${DELTA%.*}" -gt "0" ]); then
        echo -e "${GREEN}✅ $file: TDG improved${NC}"
        echo "   Baseline: $BASELINE_SCORE ($BASELINE_GRADE) → Current: $CURRENT_SCORE ($CURRENT_GRADE)"
        echo "   Delta: +$DELTA"
        IMPROVEMENTS=$((IMPROVEMENTS + 1))
    else
        echo "➖ $file: TDG stable ($CURRENT_SCORE, $CURRENT_GRADE)"
    fi
    
    # Log to history
    echo "$(date -Iseconds) | $file | $BASELINE_SCORE→$CURRENT_SCORE | $BASELINE_GRADE→$CURRENT_GRADE | $DELTA" >> "$TDG_HISTORY"
done

# Calculate file hash for change detection
CURRENT_HASH=$(sha256sum $MODIFIED_FILES 2>/dev/null | sha256sum | cut -d' ' -f1)
echo "📝 File hash: $CURRENT_HASH"

# Summary
echo ""
echo "📊 TDG Summary:"
echo "   Improvements: $IMPROVEMENTS files"
echo "   Violations: $VIOLATIONS files"

if [ $VIOLATIONS -gt 0 ]; then
    echo -e "${RED}❌ BLOCKED: TDG degradation detected${NC}"
    echo "To proceed, either:"
    echo "1. Improve the code to maintain or improve TDG score"
    echo "2. Add [TDG-OVERRIDE] to commit message with justification"
    echo "3. Update baseline with: cp $TDG_CURRENT $TDG_BASELINE"
    exit 1
fi

echo -e "${GREEN}✅ TDG check passed${NC}"
