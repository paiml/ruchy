#!/bin/bash
# Setup script for pre-commit hooks

echo "🔧 Ruchy Pre-commit Hook Setup"
echo "=============================="
echo ""
echo "Choose your pre-commit hook mode:"
echo "1. Fast (5-10s) - Critical checks only"
echo "2. Standard (30-60s) - Comprehensive validation"
echo "3. Disable - No pre-commit checks"
echo ""
read -p "Enter choice (1-3): " choice

HOOKS_DIR=".git/hooks"

case $choice in
    1)
        echo "Installing fast pre-commit hook..."
        cp "$HOOKS_DIR/pre-commit-fast" "$HOOKS_DIR/pre-commit"
        chmod +x "$HOOKS_DIR/pre-commit"
        echo "✅ Fast hooks installed (5-10s checks)"
        echo "Critical checks only - run 'make test' for full validation"
        ;;
    2)
        echo "Installing standard pre-commit hook..."
        if [ -f "$HOOKS_DIR/pre-commit.bak" ]; then
            cp "$HOOKS_DIR/pre-commit.bak" "$HOOKS_DIR/pre-commit"
        else
            echo "Error: Standard hook backup not found"
            exit 1
        fi
        chmod +x "$HOOKS_DIR/pre-commit"
        echo "✅ Standard hooks installed (30-60s comprehensive checks)"
        ;;
    3)
        echo "Disabling pre-commit hooks..."
        if [ -f "$HOOKS_DIR/pre-commit" ]; then
            mv "$HOOKS_DIR/pre-commit" "$HOOKS_DIR/pre-commit.disabled"
        fi
        echo "⚠️  Pre-commit hooks disabled"
        echo "Remember to run 'make test' manually before committing!"
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "Current hook status:"
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    if grep -q "FAST VERSION" "$HOOKS_DIR/pre-commit" 2>/dev/null; then
        echo "Mode: Fast (5-10s)"
    else
        echo "Mode: Standard (30-60s)"
    fi
else
    echo "Mode: Disabled"
fi