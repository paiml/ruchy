#!/bin/bash
set -e

# Script to publish Ruchy to crates.io
# Usage: ./scripts/publish-crates.sh [--dry-run]

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

DRY_RUN=""
if [ "$1" == "--dry-run" ]; then
    DRY_RUN="--dry-run"
    echo -e "${YELLOW}Running in dry-run mode${NC}"
fi

echo -e "${GREEN}Publishing Ruchy to crates.io${NC}"
echo "================================"

# Check if logged in to crates.io
if ! cargo login --help > /dev/null 2>&1; then
    echo -e "${RED}Error: cargo not found${NC}"
    exit 1
fi

# Check if we have a token
if [ -z "$CARGO_REGISTRY_TOKEN" ] && [ ! -f ~/.cargo/credentials.toml ]; then
    echo -e "${YELLOW}No crates.io token found.${NC}"
    echo "Please run: cargo login <YOUR_TOKEN>"
    echo "Get your token from: https://crates.io/settings/tokens"
    exit 1
fi

# Ensure we're on a clean working directory
if [ -z "$DRY_RUN" ]; then
    if [ -n "$(git status --porcelain)" ]; then
        echo -e "${RED}Error: Working directory has uncommitted changes${NC}"
        echo "Please commit or stash your changes before publishing"
        exit 1
    fi
fi

# Step 1: Publish the library crate
echo -e "\n${GREEN}Step 1: Publishing ruchy library${NC}"
echo "-----------------------------------"
cargo publish --package ruchy $DRY_RUN

if [ -z "$DRY_RUN" ]; then
    echo -e "${YELLOW}Waiting for crates.io to index the package...${NC}"
    for i in {1..60}; do
        echo -n "."
        sleep 1
    done
    echo ""
    
    # Verify the package is available
    echo -e "${GREEN}Verifying package on crates.io...${NC}"
    if curl -s https://crates.io/api/v1/crates/ruchy | grep -q '"name":"ruchy"'; then
        echo -e "${GREEN}✓ ruchy library successfully published!${NC}"
        echo "View at: https://crates.io/crates/ruchy"
    else
        echo -e "${YELLOW}⚠ Package may still be indexing${NC}"
    fi
fi

if [ -z "$DRY_RUN" ]; then
    echo -e "${GREEN}✓ ruchy successfully published!${NC}"
    echo "View at: https://crates.io/crates/ruchy"

    # Step 2: Test installation
    echo -e "\n${GREEN}Step 2: Testing installation${NC}"
    echo "-----------------------------"
    echo "You can now install Ruchy with:"
    echo -e "${YELLOW}cargo install ruchy${NC}"

    echo -e "\n${GREEN}🎉 Publication complete!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Test installation: cargo install ruchy"
    echo "2. Verify documentation: https://docs.rs/ruchy"
    echo "3. Share the news!"
    echo ""
    echo "Installation methods:"
    echo "  - Cargo: cargo install ruchy"
    echo "  - Binary: https://github.com/paiml/ruchy/releases/latest"
else
    echo -e "\n${GREEN}Dry run complete!${NC}"
    echo "To publish for real, run without --dry-run flag"
fi