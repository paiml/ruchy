#!/bin/bash
# Validation script for MCP publishing configuration

set -e

echo "🔍 Validating MCP Publishing Configuration..."
echo ""

# Check server.json exists
if [ ! -f "server.json" ]; then
    echo "❌ server.json not found"
    exit 1
fi
echo "✅ server.json exists"

# Validate JSON syntax
if ! node -e "JSON.parse(require('fs').readFileSync('server.json'))" 2>/dev/null; then
    echo "❌ server.json is not valid JSON"
    exit 1
fi
echo "✅ server.json is valid JSON"

# Extract and validate fields
IDENTIFIER=$(node -e "console.log(JSON.parse(require('fs').readFileSync('server.json')).identifier)")
VERSION=$(node -e "console.log(JSON.parse(require('fs').readFileSync('server.json')).version)")
CARGO_VERSION=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version')

echo ""
echo "📋 Configuration Summary:"
echo "  Identifier: $IDENTIFIER"
echo "  server.json version: $VERSION"
echo "  Cargo.toml version: $CARGO_VERSION"

# Check version sync
if [ "$VERSION" != "$CARGO_VERSION" ]; then
    echo ""
    echo "⚠️  WARNING: Version mismatch between server.json and Cargo.toml"
    echo "   Update server.json version to match Cargo.toml: $CARGO_VERSION"
fi

# Check identifier format
if ! echo "$IDENTIFIER" | grep -qE '^io\.github\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+$'; then
    echo ""
    echo "⚠️  WARNING: Identifier should follow format: io.github.username.project"
fi

# Check workflow file
if [ ! -f ".github/workflows/publish-mcp.yml" ]; then
    echo ""
    echo "❌ GitHub Actions workflow not found"
    exit 1
fi
echo ""
echo "✅ GitHub Actions workflow exists"

# Check for required secrets documentation
echo ""
echo "📝 Required GitHub Secrets:"
echo "  - CARGO_TOKEN (for crates.io publishing)"
echo ""
echo "💡 To publish:"
echo "  1. Automated: git tag v$CARGO_VERSION && git push origin v$CARGO_VERSION"
echo "  2. Manual: See MCP_PUBLISHING.md for instructions"
echo ""
echo "✅ All validation checks passed!"
