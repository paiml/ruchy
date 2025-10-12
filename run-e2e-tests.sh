#!/bin/bash
# E2E Test Runner - Uses correct Node.js path from nvm
# NOTEBOOK-009 E2E Testing

# Set Node.js path from nvm
export PATH="/home/noah/.nvm/versions/node/v22.13.1/bin:$PATH"

# Verify Node is available
if ! command -v node &> /dev/null; then
    echo "Error: Node.js not found in PATH"
    exit 1
fi

echo "Using Node.js version: $(node --version)"
echo "Using npm version: $(npm --version)"

# Run Playwright tests
npx playwright test "$@"
