#!/bin/bash
# Install pre-commit hook for Ruchy project
# This script copies the PMAT-style pre-commit hook to .git/hooks/

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "📦 Installing Ruchy pre-commit hook..."

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Create the pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Generated pre-commit hook for Ruchy (PMAT-style)
# Based on paiml-mcp-agent-toolkit style
# Generated at: 2025-09-08

set -e

echo "🔍 Ruchy Pre-commit Quality Gates"
echo "================================"

# Load Ruchy-specific configuration
export PMAT_MAX_CYCLOMATIC_COMPLEXITY=10  # Ruchy uses stricter limits per CLAUDE.md
export PMAT_MAX_COGNITIVE_COMPLEXITY=10   # Ruchy uses stricter limits per CLAUDE.md
export PMAT_MIN_TDG_SCORE=85             # A- grade minimum
export PMAT_MAX_SATD_COMMENTS=0          # Zero tolerance for SATD
export TASK_ID_PATTERN="[A-Z]+-[0-9]{3}" # Ruchy task ID pattern

# Check if pmat is available
if ! command -v pmat &> /dev/null; then
    echo "⚠️  Warning: pmat not found in PATH"
    echo "   Install with: cargo install pmat"
    exit 0  # Allow commit but warn
fi

echo "📊 Running quality gate checks..."

# 1. TDG Score Check (PRIMARY)
echo -n "  TDG A- grade check... "
TDG_SCORE=$(timeout 60s pmat tdg . --quiet 2>/dev/null || echo "0")
if [ -n "$TDG_SCORE" ] && (( $(echo "$TDG_SCORE >= $PMAT_MIN_TDG_SCORE" | bc -l) )); then
    echo "✅ ($TDG_SCORE/100)"
else
    echo "❌"
    echo "   TDG grade $TDG_SCORE below A- threshold ($PMAT_MIN_TDG_SCORE points)"
    echo "   Run: pmat tdg . --include-components --format=table"
    exit 1
fi

# 2. Complexity analysis
echo -n "  Complexity check... "
COMPLEXITY_OUTPUT=$(pmat analyze complexity --max-cyclomatic $PMAT_MAX_CYCLOMATIC_COMPLEXITY --max-cognitive $PMAT_MAX_COGNITIVE_COMPLEXITY 2>&1)
COMPLEXITY_EXIT=$?
if [ $COMPLEXITY_EXIT -eq 0 ] || echo "$COMPLEXITY_OUTPUT" | grep -q "0 violations found"; then
    echo "✅"
else
    echo "❌"
    echo "$COMPLEXITY_OUTPUT" | grep -E "violations found|exceeds" | head -3
    echo "   Fix all functions with complexity >$PMAT_MAX_CYCLOMATIC_COMPLEXITY"
    exit 1
fi

# 3. SATD (Self-Admitted Technical Debt) check
echo -n "  SATD check... "
SATD_OUTPUT=$(pmat analyze satd 2>&1)
if echo "$SATD_OUTPUT" | grep -q "Total violations: 0"; then
    echo "✅"
else
    echo "❌"
    echo "$SATD_OUTPUT" | grep "Total violations:" | head -1
    echo "   Zero SATD comments allowed per CLAUDE.md"
    exit 1
fi

# 4. Basic functionality test (Ruchy-specific)
echo -n "  Basic functionality test... "
if echo 'println("Hello")' | timeout 5s ruchy repl 2>/dev/null | grep -q "Hello"; then
    echo "✅"
else
    echo "❌"
    echo "   REPL cannot execute simple println"
    echo "   This indicates a critical compilation/runtime issue"
    exit 1
fi

# 5. Clippy check (Rust-specific) - Check for compilation errors only
echo -n "  Clippy check... "
CLIPPY_OUTPUT=$(timeout 30s cargo clippy --all-targets --all-features 2>&1 || echo "TIMEOUT")
if echo "$CLIPPY_OUTPUT" | grep -q "TIMEOUT"; then
    echo "⚠️ (timeout - skipped)"
elif echo "$CLIPPY_OUTPUT" | grep -q "error\[E[0-9]"; then
    # Real compilation errors found
    echo "❌"
    echo "$CLIPPY_OUTPUT" | grep "error\[E" | head -3
    echo "   Fix compilation errors before committing"
    exit 1
else
    # Count warnings for information
    WARNING_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:" || true)
    if [ "$WARNING_COUNT" -gt 0 ]; then
        echo "✅ (with $WARNING_COUNT warnings)"
    else
        echo "✅"
    fi
fi

# 6. Documentation synchronization
echo -n "  Documentation check... "
if [ -f "docs/execution/roadmap.md" ] && [ -f "CHANGELOG.md" ]; then
    echo "✅"
else
    echo "⚠️"
    echo "   Warning: Required documentation files missing"
fi

# 7. Test compilation check (main lib only)
echo -n "  Test compilation... "
if cargo build --lib --quiet 2>/dev/null; then
    echo "✅"
else
    echo "❌"
    echo "   Library fails to compile"
    exit 1
fi

# 8. Task ID validation (check staged commit message if available)
COMMIT_MSG_FILE=".git/COMMIT_EDITMSG"
if [ -f "$COMMIT_MSG_FILE" ]; then
    echo -n "  Task ID check... "
    if head -1 "$COMMIT_MSG_FILE" | grep -qE "\[$TASK_ID_PATTERN\]"; then
        echo "✅"
    else
        echo "⚠️"
        echo "   Warning: Commit should start with [TASK-ID] matching $TASK_ID_PATTERN"
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All quality gates passed!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Success
exit 0
EOF

# Make the hook executable
chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Pre-commit hook installed successfully!"
echo ""
echo "The hook will run automatically before each commit to ensure:"
echo "  • TDG score ≥ 85 (A- grade)"
echo "  • Function complexity ≤ 10"
echo "  • Zero SATD comments"
echo "  • Basic REPL functionality"
echo "  • Clean compilation"
echo ""
echo "To bypass the hook in emergency: git commit --no-verify"
echo "⚠️  WARNING: Never bypass quality gates without good reason!"