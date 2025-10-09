#!/bin/bash
#
# Install WASM Quality Git Hooks
#
# This script installs pre-commit and pre-push hooks that enforce
# WASM quality gates before committing and pushing code.
#

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🔧 Installing WASM Quality Git Hooks${NC}"
echo "===================================="
echo ""

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo -e "${YELLOW}⚠️  Not in a git repository root directory${NC}"
    echo "Please run this script from the Ruchy project root."
    exit 1
fi

# Check if hooks directory exists
if [ ! -d ".git/hooks" ]; then
    mkdir -p .git/hooks
    echo "Created .git/hooks directory"
fi

# Install pre-commit hook
echo "📝 Installing pre-commit hook..."
if [ -f ".git/hooks/pre-commit" ]; then
    echo -e "${YELLOW}⚠️  Existing pre-commit hook found${NC}"
    read -p "Overwrite? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipping pre-commit hook installation"
    else
        ln -sf ../../scripts/wasm-pre-commit.sh .git/hooks/pre-commit
        echo -e "${GREEN}✅ Pre-commit hook installed${NC}"
    fi
else
    ln -sf ../../scripts/wasm-pre-commit.sh .git/hooks/pre-commit
    echo -e "${GREEN}✅ Pre-commit hook installed${NC}"
fi

# Install pre-push hook
echo "📝 Installing pre-push hook..."
if [ -f ".git/hooks/pre-push" ]; then
    echo -e "${YELLOW}⚠️  Existing pre-push hook found${NC}"
    read -p "Overwrite? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Skipping pre-push hook installation"
    else
        ln -sf ../../scripts/wasm-pre-push.sh .git/hooks/pre-push
        echo -e "${GREEN}✅ Pre-push hook installed${NC}"
    fi
else
    ln -sf ../../scripts/wasm-pre-push.sh .git/hooks/pre-push
    echo -e "${GREEN}✅ Pre-push hook installed${NC}"
fi

# Make hooks executable
chmod +x .git/hooks/pre-commit 2>/dev/null || true
chmod +x .git/hooks/pre-push 2>/dev/null || true

echo ""
echo "===================================="
echo -e "${GREEN}✅ WASM Quality Hooks Installed!${NC}"
echo ""
echo "Hooks will now run automatically:"
echo "  - pre-commit: Quick quality checks (~3s)"
echo "  - pre-push: Full test suite (~15s)"
echo ""
echo "To bypass hooks (not recommended):"
echo "  git commit --no-verify"
echo "  git push --no-verify"
echo ""
echo "To uninstall hooks:"
echo "  rm .git/hooks/pre-commit"
echo "  rm .git/hooks/pre-push"
echo ""
